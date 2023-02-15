wit_bindgen_wasmtime::import!("../wits/wasmgatewayfunctions.wit");

use wasmgatewayfunctions::{Wasmgatewayfunctions, WasmgatewayfunctionsData};
use wit_bindgen_wasmtime::wasmtime::Store;

pub fn run_module(wasm_path: &str, wasm_config_path: &str, allowed_host: Option<String>) {
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    let wasm_funcs = super::instantiate(
        wasm_path,
        |store: &mut Store<super::Context<WasmgatewayfunctionsData>>, module, linker| {
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            super::hostobservability::add_to_linker(
                linker,
                |ctx| -> &mut super::Hostobservability { ctx.runtime_data.as_mut().unwrap() },
            )?;

            // Instantiates wasm module instance from auto generated binding code.
            let funcs =
                Wasmgatewayfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);

            Ok(funcs.unwrap().0)
        },
        || super::default_wasi(false, None),
        allowed_host,
    );

    let (wasm_exports, gateway_store) =
        wasm_funcs.expect("Could not load functions from wasm module.");

    // Call init of guest/wasm modules
    wasm_exports
        .init(gateway_store, wasm_config_path)
        .expect("Could not call the function.");    
}
