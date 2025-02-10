use swayipc::Connection;

use crate::config::Config;
use crate::sway_commands::{focus_workspace, get_active_monitors, get_workspaces, move_container};
use super::utils::{find_focused_workspace, get_target_index};
use super::workspace_id::WorkspaceId;


pub fn move_container_to_workspace_group(
    connection: &mut Connection,
    config: &Config,
    destination: &str,
    monitor_group: Option<&String>,
    change_focus: bool,
) {
    let workspaces = get_workspaces(connection);

    let focused_workspace = find_focused_workspace(&workspaces);
    
    let focused_workspace_id = WorkspaceId::parse_safe(&focused_workspace.name);
    let (target_monitor_index, default_monitor_group) = focused_workspace_id.map(|id| {
        (
            id.get_monitor_index(),
            id.get_monitor_group_name().to_string(),
        )
    }).unwrap_or_else(|| {
        // focused workspace is not managed, get main monitor of primary group
        let active_monitors = get_active_monitors(connection);
        let monitor_group = config.get_primary_group();
        (
            monitor_group.get_main_monitor_index(&active_monitors),
            monitor_group.name.clone(),
        )
    });
    let target_monitor_group = monitor_group.unwrap_or(&default_monitor_group);

    let target_group_index = get_target_index(&workspaces, target_monitor_group, destination);

    let workspace_id = WorkspaceId::new(
        target_monitor_group,
        target_monitor_index,
        target_group_index,
    );

    move_container(connection, &workspace_id);

    if change_focus {
        focus_workspace(connection, &workspace_id);
    };
}

