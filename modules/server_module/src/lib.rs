// Export WIT bindings in Rust lang from wasm module to be used by host code.
wit_bindgen_rust::export!("../../wits/wasmserverfunctions.wit");

mod server_config;
use std:: {fs, error::Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use anyhow::{Result};

const MODULE_NAME: &str = "Pub-Sub Messaging Module [Echo Server Currently]";

struct Wasmserverfunctions;
impl wasmserverfunctions::Wasmserverfunctions for Wasmserverfunctions {

    // Initialise module with required configuration.
    fn init(config_file_path: String, preopended_socket_fd: u32) // -> Result<()>
    {
        println!("Initialising server module '{}'", MODULE_NAME);

        let configfilepath = config_file_path;
        let configfilecontent = fs::read_to_string(configfilepath).unwrap();

        let server_config = server_config::Configuration::new(configfilecontent);
        let iot_edge_connection_string = server_config.connection_string();
        
        println!("Loaded configuration {}", iot_edge_connection_string);        
        
        server(preopended_socket_fd).unwrap()
        // Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn server(fd: u32) -> Result<(), Box<dyn Error>> {
    let listener = get_tcplistener(fd).await?;
    
    loop {
        // Asynchronously wait for an inbound socket.
        let stream_res = listener.accept().await;

        if let Err(e) = stream_res {
            println!("failed to accept connection; error = {}", e);
            continue;
        }

        let (socket, _) = stream_res.unwrap();
        println!("Connection received.");
        
        tokio::spawn(async move {
            if let Err(e) = process(socket).await {
                println!("failed to process connection; error = {}", e);
            }
        });
    }
}

#[cfg(target_os = "wasi")]
async fn get_tcplistener(fd: u32) -> Result<TcpListener> {
    use std::os::wasi::io::FromRawFd;
    
    // use file descriptor passed in for the preopened socket,
    // this must match in the calling host's WASI configuration.
    let stdlistener = unsafe { std::net::TcpListener::from_raw_fd(fd.try_into().unwrap()) };
    stdlistener.set_nonblocking(true)?;
    Ok(TcpListener::from_std(stdlistener)?)
}

async fn process(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        
        let n = socket.read(&mut buf).await.expect("failed to read data from socket");

        if n == 0 {
            return Ok(());
        }
        
        socket.write_all(&buf[0..n]).await.expect("failed to write data to socket");
    }
}