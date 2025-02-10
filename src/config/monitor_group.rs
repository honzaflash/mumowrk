use serde::{Serialize, Deserialize};
use std::collections::HashSet;


/// The index of the origin workspace group
pub const FIRST_WORKSPACE_GROUP: i32 = 1;

/// Group of monitors that should share a workspace group
#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorGroup {
    pub name: String,
    pub monitors: Vec<String>,
}


impl MonitorGroup {
    #[allow(dead_code)]
    /// Creates a new [`MonitorGroup`].
    pub fn new(name: &str, monitors: Vec<String>) -> Self {
        return Self {
            name: name.to_string(),
            monitors,
        };
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn get_main_monitor_index(&self, active_monitors: &HashSet<String>) -> usize {
        self.monitors.iter()
            .position(|monitor| active_monitors.contains(monitor))
            .expect("No active monitors in the primary monitor group")
    }

    pub fn get_monitor_index(&self, monitor_name: &str) -> Option<usize> {
        self.monitors.iter()
            .position(|monitor| monitor == monitor_name)
    }
}