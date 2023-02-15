wit_bindgen_rust::import!("../wits/hostobservability.wit");
wit_bindgen_rust::export!("../wits/wasmtelemetryfunctions.wit");

mod config;

use rand::Rng;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

const MODULE_NAME: &str = "Simulated Telemetry";

struct Wasmtelemetryfunctions;

impl wasmtelemetryfunctions::Wasmtelemetryfunctions for Wasmtelemetryfunctions {
    // Initialise module with required configuration.
    #[tokio::main(flavor = "current_thread")]
    async fn init(config_file_path: String) {
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

            hostobservability::publish("telemetry", &telemetry_message);

            // Wait for configured time.
            sleep(Duration::from_millis(
                telemetry_interval_in_milliseconds.into(),
            ));
        }
    }
}
