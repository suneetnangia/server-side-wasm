use std::{time};
use bytes::Bytes;

// Posts message to IoT Hub over Https
// TODO: Add parameters for device url/connection string
#[no_mangle]
pub extern "C" fn post_iot_message(param1: i32, param2: i32) {

    println!("Param1: {:?}, Param2: {:?}", param1, param2);
    
    let mut counter = 0;

    loop {
        let url = "https://iothubsmn.azure-devices.net/devices/rusty_device01/messages/events?api-version=2020-03-13".to_string();
        let req = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(&url)
            .header("Authorization", "SharedAccessSignature sr=iothubsmn.azure-devices.net%2Fdevices%2Frusty_device01&sig=Ybeo4ljH4fiEJkV%2Brq2%2BoumaeL5mcrTIeCSmQ889FoQ%3D&se=1656373155")
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
