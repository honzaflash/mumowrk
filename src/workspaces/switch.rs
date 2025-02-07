use itertools::Itertools;
use regex::Regex;
use swayipc::Connection;

use crate::config::Config;
use super::workspace_id::WorkspaceId;


pub fn switch_workspace_groups(connection: &mut Connection, config: &Config, monitor_group: &str, destination: &str) {
    let workspaces = connection.get_workspaces().expect("Failed to get workspaces");

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

    // Find the workspace that should be in focus after the switch
    let focused_workspace = workspaces.iter()
        .find(|workspace| workspace.focused)
        .expect("No focused workspace");
    let focused_workspace_id = WorkspaceId::parse_safe(&focused_workspace.name);
    let monitor_index_to_focus = focused_workspace_id.and_then(|id| {
        // focused workspace is managed, was able to parse the ID
        if id.get_monitor_group_name() == monitor_group {
            // the focused workspace is in the target group
            Some(id.get_monitor_index())
        } else {
            // the focused workspace is not in the target group
            None
        }
    }).or(
        // focused workspace is not managed, find the index of the monitor if
        // it is part of the target monitor group to switch to a managed workspace
        config.get_group(monitor_group).and_then(
            |group| group.get_monitor_index(&focused_workspace.output)
        )
    );
    // If the monitor index to focus is None, keep the original focus
    let next_focus = monitor_index_to_focus
        .map(|focused_monitor_index|
            WorkspaceId::new(monitor_group, focused_monitor_index,next_index).to_string()
        ).unwrap_or(focused_workspace.name.clone());

    // Switch the workspaces
    let group_config = config.groups.iter().find(|group| group.name == monitor_group)
        .expect("Monitor group not found");
    let commands = group_config.monitors.iter()
        .enumerate()
        .map(|(monitor_index, monitor)| {
            let workspace_id = WorkspaceId::new(monitor_group, monitor_index, next_index);
            format!("workspace {} output {}; workspace {};", workspace_id, monitor, workspace_id)
        })
        .join("");
    connection.run_command(commands).expect("Failed to switch workspace group");

    // Focus the workspace that should be in focus
    connection.run_command(
        format!("workspace {}", next_focus)
    ).expect("Failed to focus workspace");
}
