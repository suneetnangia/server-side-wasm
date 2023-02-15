// Export WIT bindings in Rust lang from wasm module to be used by host code.
wit_bindgen_rust::import!("../wits/hostobservability.wit");
wit_bindgen_rust::export!("../wits/wasmserverfunctions.wit");

mod config;

use anyhow::Result;
use queues::*;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::sleep;

const MODULE_NAME: &str = "Psuedo Pub-Sub Messaging";

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, val: String },
}

struct Wasmserverfunctions;

impl wasmserverfunctions::Wasmserverfunctions for Wasmserverfunctions {
    // Initialise module with required configuration.
    fn init(config_file_path: String, preopened_socket_fd: u32) {
        let config_file_content = fs::read_to_string(config_file_path).unwrap();
        let server_config = config::Configuration::new(config_file_content);
        let receiver_loop_interval_in_milliseconds =
            server_config.receiver_loop_interval_in_milliseconds();
        let data_read_buffer_size = server_config.data_read_buffer_size();

        hostobservability::loginfo(
            MODULE_NAME,
            &format!("Initialising module with: file descriptor: '{preopened_socket_fd}', read buffer size: '{data_read_buffer_size}', topic queues: '{:?}'", server_config.topics()));

        // Pre-create topics here from configuration for now, and make it dynamic later.
        let mut topics: HashMap<String, Queue<String>> = HashMap::new();
        for topic in server_config.topics() {
            let queue: Queue<String> = queue![];
            topics.insert(topic, queue);
        }

        // Starts server on the pre-opened socket provided by WASI
        run_server(
            preopened_socket_fd,
            data_read_buffer_size,
            receiver_loop_interval_in_milliseconds,
            topics,
        )
        .unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn run_server(
    fd: u32,
    data_read_buffer_size: u32,
    receiver_loop_interval_in_milliseconds: u64,
    mut topics: HashMap<String, Queue<String>>,
) -> Result<()> {
    let listener = get_tcplistener(fd).await?;

    let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel::<Command>();
    // Create a broadcast channel (1:many) which can serve as pipe for sending susbcription cmd response to connections.
    let (cmd_response_sender, _cmd_response_receiver) = broadcast::channel::<(String, String)>(2);
    let cmd_response_sender_clone = cmd_response_sender.clone();

    // Get/Set command receive task loop.
    tokio::task::spawn(async move {
        loop {
            for topic in &topics {
                hostobservability::loginfo(
                    MODULE_NAME,
                    &format!("Topic: {0}, Size: {1}", topic.0, topic.1.size()),
                );
            }

            while let Ok(cmd) = cmd_receiver.try_recv() {
                match cmd {
                    Command::Set { key, val } => {
                        if topics.contains_key(&key) {
                            // Store items in topic's queue
                            topics.get_mut(&key).unwrap().add(val.to_string()).unwrap();
                        }
                    }
                    Command::Get { key } => {
                        if topics.contains_key(&key) {
                            // Remove items from topic's queue
                            let queue = topics.get_mut(&key).unwrap();

                            if queue.size() > 0 {
                                let val = topics.get_mut(&key).unwrap().remove().unwrap();
                                cmd_response_sender_clone
                                    .send((key, val.to_string()))
                                    .unwrap();
                                // hostobservability::loginfo(MODULE_NAME,  &format!("Cmd response sent with value {}", val));
                            } else {
                                hostobservability::loginfo(MODULE_NAME,  &format!("Cannot retrieve element from empty queue '{key}', returning 'empty'."));
                                cmd_response_sender_clone
                                    .send((key, "empty".to_string()))
                                    .unwrap();
                            }
                        }
                    }
                }
            }

            // Wait for configured time after each receive loop iteration
            sleep(Duration::from_millis(
                receiver_loop_interval_in_milliseconds,
            ))
            .await;
        }
    });

    // Connection receive task loop.
    loop {
        // Asynchronously wait for an inbound connection.
        let stream_res = listener.accept().await;

        if let Err(e) = stream_res {
            hostobservability::loginfo(
                MODULE_NAME,
                &format!("Failed to accept connection; error = {}", e),
            );
            continue;
        }

        let (stream, _addr) = stream_res?;

        // hostobservability::loginfo(MODULE_NAME, &format!("Connection received on a preopened socket address {0:?}", addr));

        // Clone sender so it can be used by a separate task.
        let cmd_sender_clone = cmd_sender.clone();
        let cmd_response_subscriber = cmd_response_sender.subscribe();

        tokio::task::spawn(async move {
            if let Err(e) = process(
                stream,
                data_read_buffer_size,
                cmd_sender_clone,
                cmd_response_subscriber,
            )
            .await
            {
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
    mut stream: TcpStream,
    data_read_buffer_size: u32,
    cmd_sender: UnboundedSender<Command>,
    mut cmd_response_subscriber: broadcast::Receiver<(String, String)>,
) -> Result<()> {
    loop {
        let mut buf = vec![0; data_read_buffer_size.try_into()?];
        let n = stream.read(&mut buf).await?;
        buf.truncate(n); // truncate any additional bytes from buffer vector.

        if n == 0 {
            hostobservability::loginfo(MODULE_NAME, "Connection dropped.");
            return Ok(());
        }

        let mut buf_str = std::str::from_utf8(&buf)?.to_string();
        // Split only first word as command or topic name and rest as value for that.
        let split_index = buf_str.find(' ').unwrap();
        let split_buf_str = buf_str.split_at_mut(split_index);

        let cmd_topic = split_buf_str.0.trim();
        let cmd_topic_value = split_buf_str.1.trim();

        if "read".eq(cmd_topic) {
            hostobservability::loginfo(
                MODULE_NAME,
                &format!("Cmd '{cmd_topic} {cmd_topic_value}' received by connection task."),
            );

            cmd_sender.send(Command::Get {
                key: cmd_topic_value.to_string(),
            })?;

            // Wait for response to be received.
            // This can be optimised as it takes 4-5 secs to respond here.
            let (key, val) = cmd_response_subscriber.recv().await?;

            // Write back read command response.
            let formatted_response = format!("{key} {val}");
            stream.write_all(formatted_response.as_bytes()).await?;
            stream.flush().await?;

            hostobservability::loginfo(
                MODULE_NAME,
                &format!(
                    "Cmd response received by connection task: '{:?}'",
                    formatted_response
                ),
            );
        } else {
            cmd_sender.send(Command::Set {
                key: cmd_topic.to_string(),
                val: cmd_topic_value.to_string(),
            })?;
        }
    }
}
