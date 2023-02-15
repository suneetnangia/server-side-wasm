use toml::Value;

pub struct Configuration {
    config_value: Value,
}

impl Configuration {
    pub fn new(configfilecontents: String) -> Self {
        Self {
            config_value: configfilecontents.parse::<Value>().unwrap(),
        }
    }

    // Returns Http post url e.g. an endpoint on https://requestbin.com/
    pub fn http_post_url(&self) -> String {
        self.config_value["http_post_url"]
            .as_str()
            .unwrap()
            .to_string()
    }

    // Returns http post interval in milliseconds
    pub fn http_post_interval_in_milliseconds(&self) -> u64 {
        self.config_value["http_post_interval_in_milliseconds"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    }

    // Returns http post's expected return code
    pub fn http_post_response_code(&self) -> u16 {
        self.config_value["http_post_response_code"]
            .as_str()
            .unwrap()
            .parse::<u16>()
            .unwrap()
    }

    // Returns topic name to subscribe to
    pub fn topic(&self) -> String {
        self.config_value["topic"]
            .as_str()
            .unwrap()
            .parse::<String>()
            .unwrap()
    }
}
