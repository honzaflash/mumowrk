use rand::{Rng, distr::Alphanumeric};
use regex::Regex;
use swayipc::Workspace;

use super::workspace_id::WorkspaceId;


/// Given CLI input, return the absolute workspace group index for the target
pub(super) fn get_target_index(workspaces: &[Workspace], monitor_group: &str, destination: &str) -> i32 {
    let destination_re = Regex::new(r"([-+])?(\d+)").unwrap();
    let (maybe_sign, value) = match destination_re.captures(destination) {
        Some(caps) => (caps.get(1).map(|m| m.as_str()), caps[2].parse::<i32>().unwrap()),
        None => panic!("Invalid destination argument: '{}'", destination),
    };

    let next_index = match maybe_sign {
        None => value,
        Some(sign) => {
            // Get current state for the target monitor group
            // 1. find the first visible (managed) workspace in the monitor group
            let current_workspace = workspaces.iter()
                .find(|workspace|
                    workspace.visible && WorkspaceId::parse_safe(&workspace.name).map(
                        |id| id.get_monitor_group_name() == monitor_group
                    ).unwrap_or(false)
                );
            // 2. get the index from the workspace name
            let current_index = match current_workspace {
                Some(workspace) => WorkspaceId::parse(&workspace.name).get_index(),
                None => {
                    eprintln!("Monitor group has no visible workspaces");
                    1
                }
            };
            current_index + if sign == "-" { -value } else { value }
        }
    };

    next_index
}


pub(super) fn find_focused_workspace(workspaces: &[Workspace]) -> &Workspace {
    workspaces.iter()
        .find(|workspace| workspace.focused)
        .expect("Unexpected: No focused workspace")
}

pub(super) fn generate_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
