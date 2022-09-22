use toml::Value;

// TODO: Add anyhow Result
pub struct Configuration {
        config_value: Value,
}

impl Configuration {
    pub fn new (configfilecontents: String) -> Self
    {
        Self{                
            config_value: configfilecontents.parse::<Value>().unwrap()
        }
    }

    // Returns connection string for IoT Hub
    pub fn connection_string(&self) -> String
    {
        self.config_value["iot_hub_device_conn_string"].as_str().unwrap().to_string()
    }
}