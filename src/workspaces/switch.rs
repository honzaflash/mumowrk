use itertools::Itertools;
use swayipc::Connection;

use crate::config::Config;
use super::utils::{find_focused_workspace, get_target_index};
use super::workspace_id::WorkspaceId;


pub fn switch_workspace_groups(connection: &mut Connection, config: &Config, monitor_group: &str, destination: &str) {
    let workspaces = connection.get_workspaces().expect("Failed to get workspaces");

    let next_index = get_target_index(&workspaces, monitor_group, destination);

    // Find the workspace that should be in focus after the switch
    let focused_workspace = find_focused_workspace(&workspaces);
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
