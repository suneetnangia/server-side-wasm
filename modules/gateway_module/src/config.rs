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

    // Returns Http post url e.g. an endpoint on https://requestbin.com/
    pub fn http_post_url(&self) -> String {
        self.config_value["http_post_url"]
            .as_str()
            .unwrap()
            .to_string()
    }
}
