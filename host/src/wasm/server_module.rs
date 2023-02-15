wit_bindgen_wasmtime::import!("../wits/wasmserverfunctions.wit");

use anyhow::Result;
use wasmserverfunctions::{Wasmserverfunctions, WasmserverfunctionsData};
use wit_bindgen_wasmtime::wasmtime::Store;

pub fn run_module(
    wasm_path: &str,
    wasm_config_path: &str,
    socket_address: Option<String>,
) -> Result<()> {
    // Create type alias for store type with context generic params for import and export types.
    // Both export and import types are struct IotData
    type IotServerModuleStore = Store<super::Context<WasmserverfunctionsData>>;

    let server_funcs = super::instantiate(
        wasm_path,
        |store: &mut IotServerModuleStore, module, linker| {
            // Add wasm host functions to linker, allowing them to be used in wasm modules.
            super::hostobservability::add_to_linker(
                linker,
                |ctx| -> &mut super::Hostobservability {
                    match ctx.runtime_data.as_mut() {
                        Some(r) => r,
                        None => todo!(),
                    }
                },
            )?;

            // Instantiates wasm module instance from auto generated binding code.
            let module =
                Wasmserverfunctions::instantiate(store, module, linker, |cx| &mut cx.exports);

            Ok(module?.0)
        },
        || super::default_wasi(true, socket_address),
        None,
    );

    let (server_exports, server_store) =
        server_funcs.expect("Could not load functions from wasm module.");

    // Call init of guest/wasm modules
    server_exports
        .init(server_store, wasm_config_path, super::PREOPENED_SOCKET_FD)
        .expect("Could not call the function.");
    
    Ok(())
}
