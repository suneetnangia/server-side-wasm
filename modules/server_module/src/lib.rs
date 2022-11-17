// Export WIT bindings in Rust lang from wasm module to be used by host code.
wit_bindgen_rust::import!("../../wits/hostobservability.wit");
wit_bindgen_rust::export!("../../wits/wasmserverfunctions.wit");

mod config;

use anyhow::Result;
use queues::*;
use std::{error::Error, fs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const MODULE_NAME: &str = "Psuedo Pub-Sub Messaging Module";

struct Wasmserverfunctions;

impl wasmserverfunctions::Wasmserverfunctions for Wasmserverfunctions {
    // Initialise module with required configuration.
    fn init(config_file_path: String, preopened_socket_fd: u32) {
        let message_queue: Queue<String> = queue![];

        let config_file_content = fs::read_to_string(config_file_path).unwrap();
        let server_config = config::Configuration::new(config_file_content);
        let data_read_buffer_size = server_config.data_read_buffer_size();

        hostobservability::loginfo(MODULE_NAME, &format!("Initialising module with file descriptor '{preopened_socket_fd}' and read buffer size '{data_read_buffer_size}'"));

        // starts echo server on the pre-opened socket provided by WASI
        run_server(preopened_socket_fd, data_read_buffer_size, message_queue).unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn run_server(
    fd: u32,
    data_read_buffer_size: u32,
    _message_queue: Queue<String>,
) -> Result<(), Box<dyn Error>> {
    let listener = get_tcplistener(fd).await?;

    loop {
        // Asynchronously wait for an inbound connection.
        let stream_res = listener.accept().await;

        if let Err(e) = stream_res {
            hostobservability::loginfo(
                MODULE_NAME,
                &format!("failed to accept connection; error = {}", e),
            );
            continue;
        }

        let (socket, addr) = stream_res?;

        hostobservability::loginfo(
            MODULE_NAME,
            &format!("Connection received on a preopened socket address {addr:?}"),
        );

        // TODO: do not move message_queue object, use a shared queue which can deal with multi writes/read in thread safe manner.
        tokio::spawn(async move {
            let message_queue: Queue<String> = queue![];
            if let Err(e) = process(socket, data_read_buffer_size, message_queue).await {
                hostobservability::loginfo(
                    MODULE_NAME,
                    &format!("failed to process connection; error = {}", e),
                );
            }
        });
    }
}

async fn get_tcplistener(fd: u32) -> Result<TcpListener> {
    use std::os::wasi::io::FromRawFd;

    // Use file descriptor passed in for the preopened socket, this must match in the calling host's WASI configuration.
    let stdlistener = unsafe { std::net::TcpListener::from_raw_fd(fd.try_into()?) };
    stdlistener.set_nonblocking(true)?;

    Ok(TcpListener::from_std(stdlistener)?)
}

async fn process(
    mut socket: TcpStream,
    data_read_buffer_size: u32,
    mut message_queue: Queue<String>,
) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; data_read_buffer_size.try_into()?];

    loop {
        let n = socket.read(&mut buf).await?;

        if n == 0 {
            return Ok(());
        }

        message_queue.add("item added".to_string());
        socket.write_all(&buf[0..n]).await?
    }
}
