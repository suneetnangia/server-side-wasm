use std::{time};
use bytes::Bytes;
use std::fs;
use toml::Value;

// Posts message to IoT Hub over Https
#[no_mangle]
pub extern "C" fn post_iot_message() {

    // Read .toml file contents for device config, dir for this file must be pre-opened by WASI.
    let filename = "./iot_edge_module_simulated.toml";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let value = contents.parse::<Value>().unwrap();

    let mut counter = 0;
    loop {
        let url = value["iot_hub_device_url"].as_str().unwrap();
        let req = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(url)
            .header("Authorization", value["iot_hub_device_conn_string"].as_str().unwrap())
            .header("Content-Type", "application/text");

        let b = Bytes::from(format!("Wasm sent a message at {:?}!", time::SystemTime::now()));
        let req = req.body(Some(b)).unwrap();

        let mut res = wasi_experimental_http::request(req).expect("cannot make post request");
        let _ = std::str::from_utf8(&res.body_read_all().unwrap())
            .unwrap()
            .to_string();
        
        counter += 1;
        println!("Sending event...{}", counter);

        assert_eq!(res.status_code, 204);
    }
}
