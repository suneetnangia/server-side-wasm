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

    // Returns delay in milliseconds for message receiver/read loop
    pub fn receiver_loop_interval_in_milliseconds(&self) -> u64 {
        self.config_value["data_read_buffer_size"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    }

    pub fn topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = vec![];

        let topics_string = self.config_value["topics"].as_str().unwrap();

        for topic in topics_string.split_whitespace() {
            topics.push(topic.to_string());
        }

        topics
    }
}
