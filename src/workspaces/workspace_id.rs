use regex::Regex;


#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceId {
    monitor_group_name: String,
    monitor_index: usize,
    workspace_group_index: i32,
}

impl WorkspaceId {
    pub fn new(group_name: &str, monitor_index: usize, workspace_group_index: i32) -> Self {
        return Self {
            monitor_group_name: group_name.to_string(),
            monitor_index,
            workspace_group_index,
        };
    }

    pub fn parse_safe(name: &str) -> Option<WorkspaceId> {
        let workspace_id_re = Regex::new(r"^(\w+)-(\d+)-(-?\d+)$").unwrap();
        workspace_id_re
            .captures(name)
            .map(|caps| WorkspaceId {
                monitor_group_name: caps[1].to_string(),
                monitor_index: caps[2].parse().expect("Failed to parse monitor index"),
                workspace_group_index: caps[3].parse().expect("Failed to parse workspace group index"),
            })
    }

    pub fn parse(name: &str) -> WorkspaceId {
        match Self::parse_safe(name) {
            Some(id) => id,
            None => panic!("Invalid Workspace ID: {}", name),
        }
    }

    pub fn to_string(&self) -> String {
        return format!("{}-{}-{}", self.monitor_group_name, self.monitor_index, self.workspace_group_index);
    }

    /// Return the index of the workspace group
    pub fn get_index(&self) -> i32 {
        return self.workspace_group_index;
    }

    // Return name of the monitor group
    pub fn get_monitor_group_name(&self) -> &str {
        return &self.monitor_group_name;
    }

    // Return the index of the monitor within its group
    pub fn get_monitor_index(&self) -> usize {
        return self.monitor_index;
    }
}
impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
