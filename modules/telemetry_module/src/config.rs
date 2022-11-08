use toml::Value;

// TODO: Add anyhow Result
pub struct Configuration {
        config_value: Value,
}

impl Configuration {
    pub fn new (configfilecontents: String) ->Self
    {
        Self{                
            config_value: configfilecontents.parse::<Value>().unwrap()
        }
    }

    // Returns telemetry interval in milliseconds
    pub fn telemetry_interval_in_milliseconds(&self) -> u32
    {
        self.config_value["telemetry_interval_in_milliseconds"].as_str().unwrap().parse::<u32>().unwrap()
    }
}