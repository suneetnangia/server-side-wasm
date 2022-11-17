use toml::Value;

// TODO: Add anyhow Result
pub struct Configuration {
    config_value: Value,
}

impl Configuration {
    pub fn new(configfilecontents: String) -> Self {
        Self {
            config_value: configfilecontents.parse::<Value>().unwrap(),
        }
    }

    // Returns buffer size when data is read from incoming stream
    pub fn data_read_buffer_size(&self) -> u32 {
        self.config_value["data_read_buffer_size"]
            .as_str()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }
}
