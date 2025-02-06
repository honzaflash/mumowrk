use std::collections::HashSet;
use swayipc::Connection;

use crate::config::{Config, FIRST_WORKSPACE_GROUP};
use super::workspace_id::WorkspaceId;


pub fn init_workspaces(connection: &mut Connection, config: &Config) {
    if config.groups.len() == 0 {
        return;
    }

    let outputs = connection.get_outputs().expect("Failed to get outputs");
    let active_monitors: HashSet<String> = outputs.iter()
        .filter(|output| output.active)
        .map(|output| output.name.clone())
        .collect();

    // Assign every managed monitor a workspace per the configured grouping
    for group in config.groups.iter() {
        for (index, monitor) in group.monitors.iter().enumerate() {
            if !active_monitors.contains(monitor) {
                continue;
            }
            let workspace_id = WorkspaceId::new(&group.name, index, FIRST_WORKSPACE_GROUP);
            // assign the workspace to the monitor
            connection.run_command(
                format!("workspace {} output {}", workspace_id, monitor)
            ).expect("Failed to assign workspace to a monitor");
            // activate the workspace
            connection.run_command(
                format!("workspace {}", workspace_id)
            ).expect("Failed to activate workspace");
        }
    }

    // Focus the main monitor's workspace
    let main_workspace_id = WorkspaceId::new(
        &config.groups[0].name,
        config.get_primary_group().get_main_monitor_index(active_monitors),
        FIRST_WORKSPACE_GROUP,
    );
    connection.run_command(
        format!("workspace {}", main_workspace_id)
    ).expect("Failed to activate workspace");
}
