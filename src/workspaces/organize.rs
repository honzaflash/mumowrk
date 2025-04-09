use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use itertools::Itertools;
use swayipc::{Connection, Workspace};

use crate::config::{Config, MonitorGroup};
use crate::sway::commands;
use crate::workspaces::WorkspaceId;


/// Reorganize all containers and workspaces to match configuration and current
/// state of monitor configuration.
pub fn reorganize_everything(connection: &mut Connection, config: &Config) {
    // TODO:
    // - save old state
    // - focus ?

    let output_nodes = commands::get_tree(connection).nodes;
    let _ = File::create(
        shellexpand::full("~/.config/mumowrk/old_tree.json").unwrap().into_owned()
    ).map(|mut file| {
        file.write_all(serde_json::to_string(&output_nodes).unwrap().as_bytes())
    }).inspect_err(|e| eprintln!("Could not save old tree: {}", e));

    // Reorganize workspace groups for each configured monitor group
    for monitor_group in &config.groups {
        println!("Reorganize monitor group {}", monitor_group.name);
        reorganize_monitor_group(connection, config, monitor_group);
    }
}

fn reorganize_monitor_group(connection: &mut Connection, config: &Config, monitor_group: &MonitorGroup) {
    let active_monitors = commands::get_active_monitors(connection);
    // get indices of active monitors in the monitor group
    let monitor_indices: HashMap<String, usize> = active_monitors.iter()
        .filter_map(|monitor_name| {
            monitor_group.get_monitor_index(monitor_name).map(|index| (monitor_name.clone(), index))
        })
        .collect();

    let all_workspaces = commands::get_workspaces(connection);
    // group all the workspaces in the monitor group into workspace groups
    let workspace_groups = all_workspaces.iter()
        .filter_map(|workspace| {
            WorkspaceId::parse_safe(&workspace.name)
                .and_then(|id| {
                    if id.get_monitor_group_name() == monitor_group.name {
                        Some((id.get_index(), workspace))
                    } else {
                        None
                    }
                })
        })
        .into_group_map();

    for (_, workspaces) in workspace_groups {
        reorganize_workspace_group(connection, config, monitor_group, &active_monitors, &monitor_indices, &workspaces);
    }
}

const FOREIGN_MONITOR_INDEX: usize = 999;

fn get_foreign_monitor<'conf>(config: &'conf Config, active_monitors: &HashSet<String>) -> &'conf String {
    config.groups.iter()
        .flat_map(|group| group.monitors.iter())
        .find(|monitor| active_monitors.contains(*monitor))
        .expect("No active monitor found for either monitor group!")
}

fn reorganize_workspace_group(
    connection: &mut Connection,
    config: &Config,
    monitor_group: &MonitorGroup,
    active_monitors: &HashSet<String>,
    monitor_indices: &HashMap<String, usize>,
    workspaces: &Vec<&Workspace>,
) {
    // TODO: if there is an empty workspace on the correct monitor unfocus it to remove it
    // get a list of available monitors (such monitor that there is no workspace with its index in this group)
    let mut available_monitors = monitor_indices.iter()
        .filter(|(_, index)| {
            !workspaces.iter().any(|workspace| {
                let id = WorkspaceId::parse(&workspace.name);
                id.get_monitor_index() == **index
            })
        });
    let mut used_foreign_monitor = false;

    for workspace in workspaces {
        let id = WorkspaceId::parse(&workspace.name);

        if monitor_indices.get(&workspace.output) == Some(&id.get_monitor_index()) {
            // workspace is on the correct monitor
            println!("Workspace {} is already on the correct monitor", id);
            continue;
        }

        // Maybe it just needs to be reassigned to the correct monitor
        let monitor_candidate_entry = monitor_indices.iter()
            .find(|(_, index)| **index == id.get_monitor_index())
            // `or` it can go to the next available monitor
            .or(available_monitors.next())
            // `or` it can go to a monitor from a different monitor group if there are no active monitors in this group
            .or((monitor_indices.is_empty() && !used_foreign_monitor).then(|| {
                used_foreign_monitor = true;
                (get_foreign_monitor(config, active_monitors), &FOREIGN_MONITOR_INDEX)
            }));
        if let Some((name, index)) = monitor_candidate_entry {
            // just move the workspace to the correct or next available monitor
            commands::move_workspace_to_monitor(connection, &id, name);
            println!("Move workspace {} to monitor {}", id, name);
            if *index != id.get_monitor_index() {
                // rename the workspace if the monitor was not the matching one
                commands::rename_workspace(connection, &id, &WorkspaceId::new(
                    id.get_monitor_group_name(),
                    *index,
                    id.get_index(),
                ));
                println!("Also rename the workspace to have monitor index {}", index);
            }
            continue;
        }

        // All monitors already have a workspace, move all its containers to an existing workspace
        let main_monitor_index = if monitor_indices.is_empty() {
            // if using a foreign monitor the workspace there should have been already created above
            FOREIGN_MONITOR_INDEX
        } else {
            // use the main monitor of the monitor group
            monitor_group.get_main_monitor_index(&monitor_indices.keys().cloned().collect())
        };
        let workspace_tree = commands::get_workspace_tree(connection, &id)
            .expect("Could not find workspace tree for workspace that is supposed to exist");
        for container in workspace_tree.nodes {
            commands::move_container_by_id(connection, container.id, &WorkspaceId::new(
                id.get_monitor_group_name(),
                main_monitor_index,
                id.get_index(),
            ));
        }
        println!("Move all containers from workspace {} to monitor {}", id, main_monitor_index);
    }
}
