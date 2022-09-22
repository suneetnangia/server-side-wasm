wit_bindgen_wasmtime::import!("../wits/wasmserverfunctions.wit");
wit_bindgen_wasmtime::import!("../wits/wasmtelemetryfunctions.wit");
wit_bindgen_wasmtime::export!("../wits/hostfunctions.wit");

use std::{fs, net as stdnet};
use anyhow::Result;
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{Dir, net, TcpListener};
use wasmserverfunctions::{Wasmserverfunctions, WasmserverfunctionsData};
use wasmtelemetryfunctions::{Wasmtelemetryfunctions, WasmtelemetryfunctionsData};

const PREOPENED_SOCKET_FD: u32 = 4;

struct Hostfunctions {
}

impl hostfunctions::Hostfunctions for Hostfunctions {
    fn sendtelemetry(&mut self, message: &str,) -> String {
         println!("Sending telemetry with content: {}", message);
        "Returned from host".to_string()
    }
}

fn main() {
    println!("Initialising edge host service...");
    let server_module_name = "server_module";
    let telemetry_module_name = "gateway_module";

    // Select the path for wasm module
    let server_module_path = if cfg!(not(debug_assertions)) {
        format!("../modules/{0}/target/wasm32-wasi/release/{0}.wasm", server_module_name)
        } else {
        format!("../modules/{0}/target/wasm32-wasi/debug/{0}.wasm", server_module_name)
    };

    let telemetry_module_path = if cfg!(not(debug_assertions)) {
        format!("../modules/{0}/target/wasm32-wasi/release/{0}.wasm", telemetry_module_name)
        } else {
        format!("../modules/{0}/target/wasm32-wasi/debug/{0}.wasm", telemetry_module_name)
    };
    
    println!("Loading wasm edge module '{}'", &server_module_path);
    println!("Loading wasm edge module '{}'", &telemetry_module_path);

    // run_server(&server_module_path, "./server_module/config.toml");
    run_telemetry(&telemetry_module_path, "./telemetry_module/config.toml");
}

fn run_server(wasm_path: &str, wasm_config_path: &str) {    
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    type IotServerModuleStore = Store<Context<WasmserverfunctionsData>>;    
    
    let server_funcs = instantiate(wasm_path,
        |store: &mut IotServerModuleStore, module, linker| {
                
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            hostfunctions::add_to_linker(linker, |ctx| -> &mut Hostfunctions { ctx.runtime_data.as_mut().unwrap() })?;
            
            // Instantiates wasm module instance from auto generated binding code.
            let a = Wasmserverfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(a.unwrap().0)
        },
        || {default_wasi(true)},
    );

    let (server_exports, server_store) = server_funcs.expect("Could not load functions from wasm module.");    

    // Call init of server and telemetry guest/wasm modules
    server_exports.init(server_store, wasm_config_path, PREOPENED_SOCKET_FD).expect("Could not call the function.");    
    
    // println!("Returned from wasm imported function with: {:?}", response);

}

fn run_telemetry(wasm_path: &str, wasm_config_path: &str) {    
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData    
    type IotTelemetryStore = Store<Context<WasmtelemetryfunctionsData>>;
    
    let telemetry_funcs = instantiate(wasm_path,
        |store: &mut IotTelemetryStore, module, linker| {
                
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            hostfunctions::add_to_linker(linker, |ctx| -> &mut Hostfunctions { ctx.runtime_data.as_mut().unwrap() })?;
            // Instantiates wasm module instance from auto generated binding code.
            let a = Wasmtelemetryfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(a.unwrap().0)
        },
        || {default_wasi(false)},
    );

    let (telemetry_exports, telemetry_store) = telemetry_funcs.expect("Could not load functions from wasm module.");

    // Call init of server and telemetry guest/wasm modules    
    telemetry_exports.init(telemetry_store, wasm_config_path).expect("Could not call the function.");
    
    // println!("Returned from wasm imported function with: {:?}", response);

}

fn default_wasi(open_socket: bool) -> wasmtime_wasi::WasiCtx {
    // Add a directory for access from wasm as root directory.
    let dir: Dir = Dir::from_std_file(fs::File::open("../modules").expect("Could not open path"));

    if open_socket
    {        
        let stdnet_tcp_listener = stdnet::TcpListener::bind("127.0.0.1:8081").unwrap();    
        let wasi_tcp_listener =  TcpListener::from_std(stdnet_tcp_listener);   
        
        let wasi_socket = net::Socket::TcpListener(wasi_tcp_listener);

        wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .preopened_dir(dir, ".").expect("Could not preopen path")
        .preopened_socket(PREOPENED_SOCKET_FD, wasi_socket).expect("Failed to open listener")
        .build()
    }    
    else {
        wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .preopened_dir(dir, ".").expect("Could not preopen path")
        .build()
    }    
}

struct Context<E> {
    wasi: wasmtime_wasi::WasiCtx,
    pub runtime_data: Option<Hostfunctions>,    
    exports: E,
}

fn instantiate<E: Default, T>(
    wasm: &str,
    mk_exports: impl FnOnce(&mut Store<Context<E>>,&Module, &mut Linker<Context<E>>,) -> Result<T>,
    wasi_ctx: impl FnOnce() -> wasmtime_wasi::WasiCtx,
) -> Result<(T, Store<Context<E>>)> {

    let config = Config::new();
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, wasm)?;

    let mut linker = Linker::new(&engine);
        
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<E>| &mut cx.wasi)?;


    
    let http = wasi_experimental_http_wasmtime::HttpCtx { allowed_hosts: Some(vec!["https://eogagdcq6w5hak.m.pipedream.net".to_string()]),
     max_concurrent_requests: Some(42) };
     
     wasi_experimental_http_wasmtime::HttpState::new()
            .expect("HttpState::new failed")
            .add_to_linker(&mut linker, move |ctx| http.clone())?;


    
    let mut store = Store::new(
        &engine,
        Context {
            wasi: wasi_ctx(),
            runtime_data: Some(Hostfunctions { }),
            exports: E::default(),
        },
    );

    let exports = mk_exports(&mut store, &module, &mut linker)?;
    
    Ok((exports, store))
}
