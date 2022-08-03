wit_bindgen_wasmtime::import!("../wits/wasmexports.wit");
wit_bindgen_wasmtime::export!("../wits/wasmimports.wit");

use anyhow::Result;
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::Dir;

struct Wasmimports {
}

impl wasmimports::Wasmimports for Wasmimports {
    fn sendtelemetry(&mut self, message: &str,) -> String {
         println!("Sending telemetry with content: {}", message);
        "Returned from host".to_string()
    }
}

fn main() {
    println!("Initialising edge host service...");

    // Select the path for wasm module
    let path = if cfg!(not(debug_assertions)) {
        "../modules/target/wasm32-wasi/release/modules.wasm"
    } else {
        "../modules/target/wasm32-wasi/debug/modules.wasm"
    };
    
    println!("Loading wasm edge module '{}'", path);
    run(path, "config.toml");
}

fn run(wasm_path: &str, wasm_config_path: &str) {
    use wasmexports::{Wasmexports, WasmexportsData};
   
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    type IotStore = Store<Context<WasmexportsData>>;
    
    let funcs = instantiate1(wasm_path,
        |store: &mut IotStore, module, linker| {
                
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            wasmimports::add_to_linker(linker, |ctx| -> &mut Wasmimports { ctx.runtime_data.as_mut().unwrap() })?;
            // Instantiates wasm module instance from auto generated binding code.
            let a = Wasmexports::instantiate(store, module, linker, |cx| &mut cx.exports);
            
            Ok(a.unwrap().0)
        }
    );

    let (exports, store) = funcs.expect("Could not load functions from wasm module.");
    let response = exports.init(store, wasm_config_path).expect("Could not call the function.");
    
    println!("Returned from wasm imported function with: {:?}", response);

}

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    // Add a directory for access from wasm as root directory.
    let dir: Dir = Dir::from_std_file(std::fs::File::open("../modules").expect("Could not open path"));

    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
         .preopened_dir(dir, ".").expect("Could not preopen path")
        .build()
}

struct Context<E> {
    wasi: wasmtime_wasi::WasiCtx,
    pub runtime_data: Option<Wasmimports>,    
    exports: E,
}

fn instantiate1<E: Default, T>(
    wasm: &str,    
    mk_exports: impl FnOnce(&mut Store<Context<E>>,&Module, &mut Linker<Context<E>>,) -> Result<T>,
) -> Result<(T, Store<Context<E>>)> {

    let config = Config::new();
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, wasm)?;

    let mut linker = Linker::new(&engine);
        
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<E>| &mut cx.wasi)?;

    let mut store = Store::new(
        &engine,
        Context {
            wasi: default_wasi(),
            runtime_data: Some(Wasmimports { }),
            exports: E::default(),
        },
    );

    let exports = mk_exports(&mut store, &module, &mut linker)?;
    
    Ok((exports, store))
}
