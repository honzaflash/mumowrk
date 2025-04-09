use swayipc::Connection;

use crate::{config::{Config, FIRST_WORKSPACE_GROUP}, sway::commands::{assign_workspace_to_monitor, focus_workspace, get_active_monitor_names}};
use super::workspace_id::WorkspaceId;


pub fn init_workspaces(connection: &mut Connection, config: &Config) {
    if config.groups.len() == 0 {
        return;
    }

    let active_monitors = get_active_monitor_names(connection);

    // Assign every managed monitor a workspace per the configured grouping
    for group in config.groups.iter() {
        for (index, monitor) in group.monitors.iter().enumerate() {
            if !active_monitors.contains(monitor) {
                continue;
            }
            let workspace_id = WorkspaceId::new(&group.name, index, FIRST_WORKSPACE_GROUP);
            assign_workspace_to_monitor(connection, &workspace_id, monitor);
            // activate the workspace on the monitor
            focus_workspace(connection, &workspace_id);
        }
    }

    // Focus the main monitor's workspace
    let main_workspace_id: WorkspaceId = WorkspaceId::new(
        &config.groups[0].name,
        config.get_primary_group().get_main_monitor_index(&active_monitors),
        FIRST_WORKSPACE_GROUP,
    );
    focus_workspace(connection, &main_workspace_id);
}
