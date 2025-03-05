use std::collections::HashSet;

use swayipc::{Connection, Workspace};

// TODO use macros for these at some point?
//   at least for the commands formatting


/// Get list of current workspaces from the sway IPC connection
/// 
/// # Panincs
/// Panics if the request fails
pub fn get_workspaces(connection: &mut Connection) -> Vec<Workspace> {
    connection.get_workspaces().expect("Failed to get workspaces")
}

/// Get list of active outputs from the sway IPC connection
/// 
/// # Panincs
/// Panics if the request fails
pub fn get_active_monitors(connection: &mut Connection) -> HashSet<String> {
    let outputs = connection.get_outputs().expect("Failed to get outputs");
    outputs.iter()
        .filter(|output| output.active)
        .map(|output| output.name.clone())
        .collect()
}

/// Run a `workspace` command over the sway IPC connection
/// to activate a workspace.
/// 
/// # Panincs
/// Panics if the command fails
pub fn focus_workspace<Id: std::fmt::Display>(connection: &mut Connection, workspace_id: &Id) {
    connection.run_command(
        format!("workspace {}", workspace_id)
    ).expect("Failed to activate workspace");
}

/// Run a `rename workspace OLD to NEW` command over the sway IPC connection
/// 
/// # Panincs
/// Panics if the command fails
pub fn rename_workspace<IdA: std::fmt::Display, IdB: std::fmt::Display>(connection: &mut Connection, old: &IdA, new: &IdB) {
    connection.run_command(
        format!("rename workspace {} to {};", old, new)
    ).expect("Failed to rename workspace");
}

/// Run a `move container to workspace` command over the sway IPC connection
/// 
/// # Panincs
/// Panics if the command fails
pub fn move_container<Id: std::fmt::Display>(connection: &mut Connection, workspace_id: &Id) {
    connection.run_command(
        format!("move container to workspace {};", workspace_id)
    ).expect("Failed to move container");
}

/// Run a `workspace` command over the sway IPC connection
/// to assign a workspace to an output
/// 
/// # Panincs
/// Panics if the command fails
pub fn assign_workspace_to_monitor<Id: std::fmt::Display>(connection: &mut Connection, workspace_id: &Id, monitor: &str) {
    connection.run_command(
        format!("workspace {} output {}", workspace_id, monitor)
    ).expect("Failed to assign workspace to a monitor");
}

/// Format a pair of commands to assign and actiate a workspace
/// on a specific monitor
pub fn get_assign_and_focus_workspace_command<Id: std::fmt::Display>(
    workspace_id: &Id,
    monitor: &str,
) -> String {
    format!("workspace {} output {}; workspace {}", workspace_id, monitor, workspace_id)
}

/// Format a pair of commands to assign and actiate a workspace
/// on a specific monitor
pub fn get_focus_workspace_command<Id: std::fmt::Display>(
    workspace_id: &Id,
) -> String {
    format!("workspace {}", workspace_id)
}
