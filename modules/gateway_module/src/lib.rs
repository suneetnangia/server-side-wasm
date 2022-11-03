wit_bindgen_rust::import!("../../wits/hostobservability.wit");
wit_bindgen_rust::export!("../../wits/wasmgatewayfunctions.wit");

mod config;

use std::{fs, time};
use rand::Rng;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use anyhow::{Result};
use bytes::Bytes;
use wasi_experimental_http as outbound_http;

pub type Request = http::Request<Option<bytes::Bytes>>;
pub type Response = http::Response<Option<bytes::Bytes>>;

const MODULE_NAME: &str = "Data Egress Gateway Module";

struct Wasmgatewayfunctions; 

impl wasmgatewayfunctions::Wasmgatewayfunctions for Wasmgatewayfunctions {

    // Initialise module with required configuration.
    fn init(config_file_path: String)
    {
        let config_file_contents = fs::read_to_string(config_file_path).unwrap();
        let config = config::Configuration::new(config_file_contents);

        hostobservability::loginfo(MODULE_NAME, &format!("Initialising module with http post url '{}'", config.http_post_url()));
       
        // Generate temperature and pressure values randomly for messages
        let mut random_number = rand::thread_rng();
        let mut message_count = 0;

        loop {

            let post_url = config.http_post_url();
            let http_request = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(post_url)                        
            .header("Content-Type", "application/text");

            let random_temp = random_number.gen_range(0.0..100.0);
            let random_pressure = random_number.gen_range(0.0..50.0);

            let message = format!("{{\"device Id\" : \"001\", \"temperature\" : {random_temp:.2}, \"pressure\":{random_pressure:.2}}}");
            let message_bytes = Bytes::from(message);
            
            message_count+=1;
            let http_request = http_request.body(Some(message_bytes)).unwrap();
            let http_response = outbound_http::request(http_request).expect("Could not make post request.");            

            hostobservability::loginfo(MODULE_NAME, &format!("Message {} sent, http status returned: {}", message_count, http_response.status_code));
            assert_eq!(http_response.status_code, 200);        

            // Uncomment this if you need to read the response's body, in our case we have an empty response
            // let http_response_body = std::str::from_utf8(&http_response.body_read_all().unwrap())
            //                         .unwrap()
            //                         .to_string();

            // Wait for 1 sec
            sleep(Duration::from_millis(1000));
        }      
    }
}