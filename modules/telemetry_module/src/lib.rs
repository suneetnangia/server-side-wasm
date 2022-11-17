wit_bindgen_rust::import!("../../wits/hostobservability.wit");
wit_bindgen_rust::export!("../../wits/wasmtelemetryfunctions.wit");

mod config;

use rand::Rng;
use std::fs;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
// use anyhow::{Result};

const MODULE_NAME: &str = "Simulated Telemetry Module";

struct Wasmtelemetryfunctions;

impl wasmtelemetryfunctions::Wasmtelemetryfunctions for Wasmtelemetryfunctions {
    // Initialise module with required configuration.
    fn init(config_file_path: String) {
        let configfilecontents = fs::read_to_string(config_file_path).unwrap();

        let telemetry_config = config::Configuration::new(configfilecontents);
        let telemetry_interval_in_milliseconds =
            telemetry_config.telemetry_interval_in_milliseconds();

        hostobservability::loginfo(
            MODULE_NAME,
            &format!(
                "Initialising module with telemetry interval of '{}' ms",
                telemetry_interval_in_milliseconds
            ),
        );

        // Generate temperature and pressure values randomly for simulation
        let mut random_number = rand::thread_rng();

        loop {
            let random_temp = random_number.gen_range(0.0..100.0);
            let random_pressure = random_number.gen_range(0.0..50.0);

            let telemetry_message = format!("{{\"device Id\" : \"001\", \"temperature\" : {random_temp:.2}, \"pressure\":{random_pressure:.2}}}");
            let time_now = SystemTime::now();

            // Start: Send telemetry to pub-sub here.
            // TODO: Flesh out code
            // End: Send telemetry to pub-sub here.

            hostobservability::loginfo(
                MODULE_NAME,
                &format!("Sent message at {time_now:?}, event: {telemetry_message}"),
            );

            // Wait for configured time.
            sleep(Duration::from_millis(
                telemetry_interval_in_milliseconds.into(),
            ));
        }
    }
}
