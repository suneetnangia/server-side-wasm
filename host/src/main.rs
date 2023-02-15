mod wasm;

use std::{thread, time::Duration};

use async_std::task;
use clap::Parser;

#[derive(Parser, Debug)]
struct CliParams {
    #[arg(short, long)]
    gateway_allowed_host: String,

    #[arg(short, long)]
    server_socket_address: String,
}

fn main() {
    let cli_params = CliParams::parse();

    println!("Initialising host service...");

    let server_module_name = "server_module";
    let gateway_module_name = "gateway_module";
    let telemetry_module_name = "telemetry_module";

    let (server_module_path, telemetry_module_path, gateway_module_path) = if cfg!(debug_assertions)
    {
        (
            format!(
                "../{0}/target/wasm32-wasi/debug/{0}.wasm",
                server_module_name
            ),
            format!(
                "../{0}/target/wasm32-wasi/debug/{0}.wasm",
                telemetry_module_name
            ),
            format!(
                "../{0}/target/wasm32-wasi/debug/{0}.wasm",
                gateway_module_name
            ),
        )
    } else {
        (
            format!(
                "../{0}/target/wasm32-wasi/release/{0}.wasm",
                server_module_name
            ),
            format!(
                "../{0}/target/wasm32-wasi/release/{0}.wasm",
                telemetry_module_name
            ),
            format!(
                "../{0}/target/wasm32-wasi/release/{0}.wasm",
                gateway_module_name
            ),
        )
    };

    println!("Server Module Path: {}", server_module_path);

    let server_task = task::spawn(async move {
        wasm::server_module::run_module(
            &server_module_path,
            &format!("./{0}/config.toml", server_module_name),
            Some(cli_params.server_socket_address),
        )
        .unwrap();
    });

    // Induce synthetic delay of 5 secs before connections can be made to server module.
    thread::sleep(Duration::from_millis(5000));

    println!("Gateway Module Path: {}", gateway_module_path);

    let gateway_task = task::spawn(async move {
        wasm::gateway_module::run_module(
            &gateway_module_path,
            &format!("./{0}/config.toml", gateway_module_name),
            Some(cli_params.gateway_allowed_host),
        );
    });

    println!("Telemetry Module Path: {}", telemetry_module_path);

    let telemetry_task = task::spawn(async move {
        wasm::telemetry_module::run_module(
            &telemetry_module_path,
            &format!("./{0}/config.toml", telemetry_module_name),
        );
    });

    task::block_on(async {
        server_task.await;
        gateway_task.await;
        telemetry_task.await;
    });
}
