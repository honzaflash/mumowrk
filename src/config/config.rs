use std::fs;

use serde::{Serialize, Deserialize};
use serde_yml;

use super::monitor_group::MonitorGroup;


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub groups: Vec<MonitorGroup>,
}

impl Config {
    pub fn load(path: &str) -> Self {
        // Load the configuration from a file
        let expanded_path = shellexpand::full(path).expect("Failed to expand path");
        let config_str = fs::read_to_string(expanded_path.into_owned()).expect("Failed to read config file");
        serde_yml::from_str(&config_str).expect("Failed to parse the config file")
    }

    pub fn get_primary_group(&self) -> &MonitorGroup {
        &self.groups[0]
    }

    pub fn get_group(&self, name: &str) -> Option<&MonitorGroup> {
        self.groups.iter().find(|group| group.name == name)
    }
}
