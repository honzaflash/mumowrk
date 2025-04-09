use std::collections::HashSet;

use swayipc::Connection;

use crate::{config::Config, sway::commands::{get_active_monitors, get_workspaces, rename_workspace}};
use super::{utils::{generate_random_string, get_target_index}, workspace_id::WorkspaceId};


// TODO: Either changes this to... or add a `fn` to...
//   take workspace group `from` and insert it at index `to` shifting all the other groups
pub fn swap_workspace_groups(
    connection: &mut Connection,
    config: &Config,
    from_index: i32,
    to: &str,
    // focus: bool,
    mon_group: Option<&String>,
) {
    let active_monitors = get_active_monitors(connection);
    let workspaces = get_workspaces(connection);
    let workspace_names: HashSet<String> = workspaces.iter()
        .map(|workspace| workspace.name.clone())
        .collect();

    let monitor_group = mon_group
        .and_then(|group_name| config.get_group(group_name))
        .unwrap_or(config.get_primary_group());
    let monitor_indices: Vec<usize> = monitor_group.monitors.iter()
        .enumerate()
        .filter(|(_, name)| active_monitors.contains(*name))
        .map(|(index, _)| index)
        .collect();
    let to_index = get_target_index(&workspaces, &monitor_group.name, to);

    let swaps: Vec<bool> = monitor_indices.iter().map(|&monitor_index| {
        let from_id = WorkspaceId::new(
            &monitor_group.name,
            monitor_index,
            from_index,
        );
        let to_id = WorkspaceId::new(
            &monitor_group.name,
            monitor_index,
            to_index,
        );
        swap_workspaces(connection, &workspace_names, &from_id.to_string(), &to_id.to_string())
    }).collect();

    if !swaps.iter().any(|b| *b) {
        println!("None of the workspaces exist. Nothing was renamed.")
    }
}

fn swap_workspaces(connection: &mut Connection, workspaces: &HashSet<String>, from: &str, to: &str) -> bool {
    // Swap the names of two workspaces and handle cases of non-existance
    if !workspaces.contains(from) {
        if !workspaces.contains(to) {
            return false;
        }
        rename_workspace(connection, &to, &from);
        return true;
    }
    
    let tmp_id = format!("tmp-{}", generate_random_string(6));
    if workspaces.contains(to) {
        rename_workspace(connection, &to, &tmp_id);
    }
    rename_workspace(connection, &from, &to);
    if workspaces.contains(to) {
        rename_workspace(connection, &tmp_id, &from);
    }
    return true;
}
