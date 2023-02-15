pub mod gateway_module;
pub mod server_module;
pub mod telemetry_module;

wit_bindgen_wasmtime::export!("../wits/hostobservability.wit");

use anyhow::Result;
use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
    task::block_on,
};
use chrono::Utc;
use std::{fs, net as stdnet};
use wasmtime_wasi::{net, Dir, TcpListener};
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Linker, Module, Store};

const PREOPENED_SOCKET_FD: u32 = 4;
const MODULE_NAME: &str = "Wasm Host";

pub struct Hostobservability;

impl hostobservability::Hostobservability for Hostobservability {
    fn loginfo(&mut self, _modulename: &str, _message: &str) {
        #[cfg(debug_assertions)]
        println!(
            "{:#?} Module: {}, Message: {}",
            Utc::now(),
            _modulename,
            _message
        );
    }

    fn publish(&mut self, topic: &str, message: &str) {
        // Publish message to pubsub server module via socket connection.
        // TODO: Reuse connection and move this to separate module.
        block_on(async {
            let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            let payload = format!("{topic} {message}");
            stream.write_all(payload.as_bytes()).await.unwrap();

            self.loginfo(MODULE_NAME, &format!("Publishing message '{payload}' to topic '{topic}' on the messaging layer via host func."));
        });
    }

    fn read(&mut self, topic: &str) -> String {
        // Read message from pubsub server module via socket connection.
        // TODO: Reuse connection and move this to separate module.
        block_on(async {
            let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            let cmd_payload = format!("read {topic}");

            self.loginfo(
                MODULE_NAME,
                &format!("Sending cmd '{cmd_payload}' on the messaging layer via host func."),
            );

            stream.write_all(cmd_payload.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();

            let mut response_buf = String::from("");

            loop {
                let mut buf = vec![0; 1000];

                let n = stream.read(&mut buf).await.unwrap();

                buf.truncate(n); // truncate any additional bytes from buffer vector.

                let buf_str = std::str::from_utf8(&buf).unwrap().to_string();
                response_buf = buf_str.clone();

                self.loginfo(
                    MODULE_NAME,
                    &format!("Received response from pubsub module: '{buf_str}'."),
                );

                // If response is received (i.e. any bytes) on socket stream, exit the loop.
                if n > 0 {
                    self.loginfo(
                        MODULE_NAME,
                        "Response stream dropped, no more data to read.",
                    );
                    break;
                }
            }

            response_buf
        })
    }

    fn subscribe(&mut self, _topic: &str) -> String {
        // TODO: Implement subscribe feature in pubsub before implementing code here.
        todo!()
    }
}

fn default_wasi(open_socket: bool, socket_address: Option<String>) -> wasmtime_wasi::WasiCtx {
    // Add a directory for access from wasm as root directory.
    let dir: Dir = Dir::from_std_file(fs::File::open("../").expect("Could not open path"));

    if open_socket {
        let stdnet_tcp_listener = stdnet::TcpListener::bind(socket_address.unwrap()).unwrap();
        let wasi_tcp_listener = TcpListener::from_std(stdnet_tcp_listener);

        let wasi_socket = net::Socket::TcpListener(wasi_tcp_listener);

        wasmtime_wasi::sync::WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(dir, ".")
            .expect("Could not preopen path")
            .preopened_socket(PREOPENED_SOCKET_FD, wasi_socket)
            .expect("Failed to open listener")
            .build()
    } else {
        wasmtime_wasi::sync::WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(dir, ".")
            .expect("Could not preopen path")
            .build()
    }
}

pub struct Context<E> {
    wasi: wasmtime_wasi::WasiCtx,
    pub runtime_data: Option<Hostobservability>,
    pub exports: E,
}

pub fn instantiate<E: Default, T>(
    wasm_path: &str,
    mk_exports: impl FnOnce(&mut Store<Context<E>>, &Module, &mut Linker<Context<E>>) -> Result<T>,
    wasi_ctx: impl FnOnce() -> wasmtime_wasi::WasiCtx,
    allowed_host: Option<String>,
) -> Result<(T, Store<Context<E>>)> {
    let config = Config::new();
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, wasm_path)?;

    let mut linker = Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<E>| &mut cx.wasi)?;

    // Only allow http outbound when requested, not all wasm modules should access it.
    let http = match allowed_host {
        None => wasi_experimental_http_wasmtime::HttpCtx {
            allowed_hosts: None,
            max_concurrent_requests: Some(42),
        },
        Some(allowed_host) => wasi_experimental_http_wasmtime::HttpCtx {
            allowed_hosts: Some(vec![allowed_host]),
            max_concurrent_requests: Some(42),
        },
    };

    // let http = wasi_experimental_http_wasmtime::HttpCtx { allowed_hosts: Some(vec![allowed_host]),max_concurrent_requests: Some(42) };

    wasi_experimental_http_wasmtime::HttpState::new()
        .expect("HttpState::new failed")
        .add_to_linker(&mut linker, move |_ctx| http.clone())?;

    let mut store = Store::new(
        &engine,
        Context {
            wasi: wasi_ctx(),
            runtime_data: Some(Hostobservability {}),
            exports: E::default(),
        },
    );

    let exports = mk_exports(&mut store, &module, &mut linker)?;

    Ok((exports, store))
}
