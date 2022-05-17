use std::{time};
use bytes::Bytes;
use libc::c_char;
use std::fs;

// Posts message to IoT Hub over Https
#[no_mangle]
pub extern "C" fn post_iot_message(_s: *const c_char) {    
    // TODO: Make use of config file content for loading device config.
    let filename = "./config.toml";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    println!("config file content:\n{}", contents);
    
    let mut counter = 0;
    loop {
        let url = "https://iothubsmn.azure-devices.net/devices/rusty_device01/messages/events?api-version=2020-03-13".to_string();
        let req = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(&url)
            .header("Authorization", "")
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
