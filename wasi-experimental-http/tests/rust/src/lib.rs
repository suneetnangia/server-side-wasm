use std::{time};

use bytes::Bytes;

// IoT Hub Post message
// TODO: Pass string parameters
#[no_mangle]
pub extern "C" fn post_iot_message(param1: i32) {

    println!("Param1: {:?}", param1);
    
    let mut counter =0i64;

    loop {
        let url = "https://iothubsmn.azure-devices.net/devices/rusty_device01/messages/events?api-version=2020-03-13".to_string();
        let req = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(&url)
            .header("Authorization", "<Replace with your device's SAS token, you can generate one from IoT Explorer client>")
            .header("Content-Type", "application/text");

        let b = Bytes::from(format!("Wasm sent a message at {:?}!", time::SystemTime::now()));
        let req = req.body(Some(b)).unwrap();

        let mut res = wasi_experimental_http::request(req).expect("cannot make post request");
        let _ = std::str::from_utf8(&res.body_read_all().unwrap())
            .unwrap()
            .to_string();
        
        counter += 1;
        println!("Sending events...{}", counter);

        assert_eq!(res.status_code, 204);
    }
}
