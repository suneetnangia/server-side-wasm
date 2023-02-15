wit_bindgen_rust::import!("../wits/hostobservability.wit");
wit_bindgen_rust::export!("../wits/wasmgatewayfunctions.wit");

mod config;

use bytes::Bytes;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use wasi_experimental_http as outbound_http;

pub type Request = http::Request<Option<bytes::Bytes>>;
pub type Response = http::Response<Option<bytes::Bytes>>;

const MODULE_NAME: &str = "Data Egress Gateway";

struct Wasmgatewayfunctions;

impl wasmgatewayfunctions::Wasmgatewayfunctions for Wasmgatewayfunctions {
    // Initialise module with required configuration.
    fn init(config_file_path: String) {
        let config_file_contents = fs::read_to_string(config_file_path).unwrap();
        let config = config::Configuration::new(config_file_contents);

        hostobservability::loginfo(
            MODULE_NAME,
            &format!(
                "Initialising module with http_post_url: {}, http_post_response_code: {}, http_post_interval_in_milliseconds: {}.",
                config.http_post_url(),
                config.http_post_response_code(),
                config.http_post_interval_in_milliseconds(),
            ),
        );

        let mut message_count = 0;

        loop {
            let post_url = config.http_post_url();
            let http_request = http::request::Builder::new()
                .method(http::Method::POST)
                .uri(post_url)
                .header("Content-Type", "application/text");
            let message = hostobservability::read(&config.topic());

            hostobservability::loginfo(
                MODULE_NAME,
                &format!("Received message {message} from pubsub."),
            );

            message_count += 1;

            let http_request = http_request.body(Some(Bytes::from(message))).unwrap();
            let http_response =
                outbound_http::request(http_request).expect("Could not make post request.");

            hostobservability::loginfo(
                MODULE_NAME,
                &format!(
                    "Message {} sent, http status returned: {}",
                    message_count, http_response.status_code
                ),
            );

            assert_eq!(http_response.status_code, config.http_post_response_code());

            // Uncomment this if you need to read the response's body, in our case we have an empty response
            // let http_response_body = std::str::from_utf8(&http_response.body_read_all().unwrap())
            //                         .unwrap()
            //                         .to_string();

            // Wait for configured time
            sleep(Duration::from_millis(
                config.http_post_interval_in_milliseconds(),
            ));
        }
    }
}
