wit_bindgen_wasmtime::import!("../wits/wasmserverfunctions.wit");
wit_bindgen_wasmtime::import!("../wits/wasmtelemetryfunctions.wit");
wit_bindgen_wasmtime::import!("../wits/wasmgatewayfunctions.wit");
wit_bindgen_wasmtime::export!("../wits/hostobservability.wit");

use anyhow::Result;
use clap::{Parser};
use std::{thread, fs, net as stdnet};
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{Dir, net, TcpListener};
use wasmserverfunctions::{Wasmserverfunctions, WasmserverfunctionsData};
use wasmtelemetryfunctions::{Wasmtelemetryfunctions, WasmtelemetryfunctionsData};
use wasmgatewayfunctions::{Wasmgatewayfunctions, WasmgatewayfunctionsData};

const PREOPENED_SOCKET_FD: u32 = 4;

#[derive(Parser)]
#[derive(Debug)]
struct CliParams
{
    #[arg(short, long)]
    gateway_allowed_host : String,

    #[arg(short, long)]
    server_socket_address : String,
}

struct Hostobservability {
}

impl hostobservability::Hostobservability for Hostobservability {
    fn loginfo(&mut self, modulename: &str, message: &str) {
         println!("Module: {}, Message: {}", modulename, message);
    }
}

fn main() {
    let cli_params = CliParams::parse();

    println!("Initialising host service...");
    
    let server_module_name = "server_module";
    let gateway_module_name = "gateway_module";
    let telemetry_module_name = "telemetry_module";

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

    let gateway_module_path = if cfg!(not(debug_assertions)) {
        format!("../modules/{0}/target/wasm32-wasi/release/{0}.wasm", gateway_module_name)
        } else {
        format!("../modules/{0}/target/wasm32-wasi/debug/{0}.wasm", gateway_module_name)
    };
    
    let server_thread = thread::spawn(move|| {
              run_server_module(&server_module_path, &format!("./{0}/config.toml", server_module_name), None, Some(cli_params.server_socket_address));
    });

    let gateway_thread = thread::spawn(move|| {
        run_gateway_module(&gateway_module_path, &format!("./{0}/config.toml", gateway_module_name), Some(cli_params.gateway_allowed_host));
    });

    let telemetry_thread = thread::spawn(move|| {
            run_telemetry_module(&telemetry_module_path, &format!("./{0}/config.toml", telemetry_module_name), None);
        });

    server_thread.join().unwrap();
    gateway_thread.join().unwrap();
    telemetry_thread.join().unwrap();

}

fn run_gateway_module (wasm_path: &str, wasm_config_path: &str, allowed_host: Option<String>) {
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    
    let wasm_funcs = 
        instantiate(wasm_path,|store: &mut Store<Context<WasmgatewayfunctionsData>>, module, linker| {
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            hostobservability::add_to_linker(linker, |ctx| -> &mut Hostobservability { ctx.runtime_data.as_mut().unwrap() })?;
            
            // Instantiates wasm module instance from auto generated binding code.
            let funcs = Wasmgatewayfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(funcs.unwrap().0)
        },

        || {default_wasi(false, None)},
        allowed_host,
    );

    let (wasm_exports, gateway_store) = wasm_funcs.expect("Could not load functions from wasm module.");

    // Call init of guest/wasm modules
    wasm_exports.init(gateway_store, wasm_config_path).expect("Could not call the function.");
    
    // println!("Returned from wasm imported function with: {:?}", response);

}

fn run_server_module(wasm_path: &str, wasm_config_path: &str, allowed_host: Option<String>, socket_address: Option<String>) {    
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    type IotServerModuleStore = Store<Context<WasmserverfunctionsData>>;
    
    let server_funcs = instantiate(wasm_path,
        |store: &mut IotServerModuleStore, module, linker| {
                
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            hostobservability::add_to_linker(linker, |ctx| -> &mut Hostobservability { ctx.runtime_data.as_mut().unwrap() })?;
            
            // Instantiates wasm module instance from auto generated binding code.
            let a = Wasmserverfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(a.unwrap().0)
        },
        || {default_wasi(true, socket_address)},
        allowed_host,
    );

    let (server_exports, server_store) = server_funcs.expect("Could not load functions from wasm module.");    

    // Call init of guest/wasm modules
    server_exports.init(server_store, wasm_config_path, PREOPENED_SOCKET_FD).expect("Could not call the function.");    
    
    // println!("Returned from wasm imported function with: {:?}", response);

}

fn run_telemetry_module (wasm_path: &str, wasm_config_path: &str, allowed_host: Option<String>) {
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    
    let wasm_funcs = 
        instantiate(wasm_path,|store: &mut Store<Context<WasmtelemetryfunctionsData>>, module, linker| {
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            hostobservability::add_to_linker(linker, |ctx| -> &mut Hostobservability { ctx.runtime_data.as_mut().unwrap() })?;
            
            // Instantiates wasm module instance from auto generated binding code.
            let funcs = Wasmtelemetryfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(funcs.unwrap().0)
        },

        || {default_wasi(false, None)},
        allowed_host,
    );

    let (wasm_exports, telemetry_store) = wasm_funcs.expect("Could not load functions from wasm module.");

    // Call init of guest/wasm modules
    wasm_exports.init(telemetry_store, wasm_config_path).expect("Could not call the function.");
    
    // println!("Returned from wasm imported function with: {:?}", response);

}

fn default_wasi(open_socket: bool, socket_address: Option<String>) -> wasmtime_wasi::WasiCtx {
    // Add a directory for access from wasm as root directory.
    let dir: Dir = Dir::from_std_file(fs::File::open("../modules").expect("Could not open path"));

    if open_socket
    {        
        let stdnet_tcp_listener = stdnet::TcpListener::bind(socket_address.unwrap()).unwrap();    
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
    pub runtime_data: Option<Hostobservability>,    
    exports: E,
}

fn instantiate<E: Default, T>(
    wasm: &str,
    mk_exports: impl FnOnce(&mut Store<Context<E>>,&Module, &mut Linker<Context<E>>,) -> Result<T>,
    wasi_ctx: impl FnOnce() -> wasmtime_wasi::WasiCtx,
    allowed_host: Option<String>,
) -> Result<(T, Store<Context<E>>)> {

    let config = Config::new();
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, wasm)?;

    let mut linker = Linker::new(&engine);
        
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<E>| &mut cx.wasi)?;
    
    // Only allow http outbound when requested, not all wasm modules should access it.
    let http = match allowed_host
    {
        None => wasi_experimental_http_wasmtime::HttpCtx { allowed_hosts: None, max_concurrent_requests: Some(42) },
        Some(allowed_host) =>wasi_experimental_http_wasmtime::HttpCtx { allowed_hosts: Some(vec![allowed_host]),max_concurrent_requests: Some(42) },
    };


    // let http = wasi_experimental_http_wasmtime::HttpCtx { allowed_hosts: Some(vec![allowed_host]),max_concurrent_requests: Some(42) };
     
     wasi_experimental_http_wasmtime::HttpState::new()
            .expect("HttpState::new failed")
            .add_to_linker(&mut linker, move |_ctx| http.clone())?;
    
    let mut store = Store::new(
        &engine,
        Context {
            wasi: wasi_ctx(),
            runtime_data: Some(Hostobservability { }),
            exports: E::default(),
        },
    );

    let exports = mk_exports(&mut store, &module, &mut linker)?;
    
    Ok((exports, store))
}
