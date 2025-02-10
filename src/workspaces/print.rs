use itertools::{Either, Itertools};
use swayipc::{Connection, EventType};

use crate::sway_commands::get_workspaces;

use super::workspace_id::WorkspaceId;


pub fn print_state_text(connection: &mut Connection) {
    let (managed, unmanaged) = get_state(connection);

    let unmanaged_workspaces_str = unmanaged.iter()
        .map(|(name, focused)| if *focused { format!("*{}*", name) } else { name.clone() } ) 
        .join(", ");

    let managed_groups_str = managed.iter()
        .map(|(mon_group, workspace_groups)| {
            let workspace_groups_str = workspace_groups.iter()
                .map(|(index, focused)| {
                    if *focused { format!("*{}*", index) } else { format!("{}", index) }
                }).join(", ");
            format!("{}: {}", mon_group, workspace_groups_str)
        })
        .join(" | ");

    let output = [
        if unmanaged_workspaces_str.is_empty() { "".to_string() } else { format!("Unmanaged: {} | ", unmanaged_workspaces_str) },
        managed_groups_str,
    ].join("");

    println!("{}", output);
}


pub fn subscribe_and_print(mut connection: Connection, printer: fn(&mut Connection)) {
    // Print initial state
    printer(&mut connection);
    // Open a new connection to listen for events
    let listen_connection = Connection::new().expect("Failed to connect to swayipc");
    let events = listen_connection.subscribe([EventType::Workspace]).expect("Failed to subscribe to workspace events");
    // Print updates as events come
    for _ in events {
        printer(&mut connection);
    }
}

/// What a waybar module expects to see
#[derive(serde::Serialize)]
struct ModuleInput {
    text: String,
    tooltip: Option<String>,
    class: Option<String>,
}

pub fn print_waybar_module(connection: &mut Connection) {
    let (managed, unmanaged) = get_state(connection);

    let unmanaged_workspaces_str = unmanaged.iter()
        .map(|(name, focused)| if *focused { format!("<u><b>{}</b></u>", name) } else { name.clone() } ) 
        .join(", ");

    let managed_groups_str = managed.iter()
        .map(|(mon_group, workspace_groups)| {
            let workspace_groups_str = workspace_groups.iter()
                .map(|(index, focused)| {
                    if *focused { format!("<u><b>{}</b></u>", index) } else { index.to_string() }
                }).join(", ");
            format!("{}: {}", mon_group, workspace_groups_str)
        })
        .join(" | ");

    let display_text = [
        if unmanaged_workspaces_str.is_empty() { "".to_string() } else { format!("({}) | ", unmanaged_workspaces_str) },
        format!("[ {} ]", managed_groups_str),
    ].join("");

    let output = ModuleInput {
        text: display_text,
        class: Some("active".to_string()),
        tooltip: Some("You can switch between these".to_string()),
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}


/// Get current workspaces and process the state into workspace groups state
fn get_state(connection: &mut Connection) -> (Vec<(String, Vec<(i32, bool)>)>, Vec<(String, bool)>) {
    let workspaces = get_workspaces(connection);
    
    let (unmanaged_workspaces, managed_ids): (Vec<_>, Vec<_>) =
        workspaces.iter().partition_map(|workspace| {
            match WorkspaceId::parse_safe(&workspace.name) {
                Some(id) => Either::Right(id),
                None => Either::Left((workspace.name.clone(), workspace.focused)),
            }
        });

    let focused_workspace_id = workspaces.iter()
        .find(|workspace| workspace.focused)
        .and_then(|workspace| WorkspaceId::parse_safe(&workspace.name));
    let workspaces_by_monitor_groups =
        managed_ids.iter().into_group_map_by(|id| id.get_monitor_group_name());
    let monitor_groups = workspaces_by_monitor_groups.iter()
        .map(|(group, workspaces)| {
            let workspace_groups = workspaces.iter()
                .unique_by(|id| id.get_index())
                .sorted_by_key(|id| id.get_index())
                .map(|id| {
                    let group_is_focused = focused_workspace_id.clone().map_or(
                        false,
                        |focused_id| focused_id.get_monitor_group_name() == *group && focused_id.get_index() == id.get_index(),
                    );
                    (id.get_index(), group_is_focused)
                }).collect();
            (*group, workspace_groups)
        })
        .sorted_by_key(|(group, _)| *group)
        .map(|(group, workspace_groups)|
            (group.to_string(), workspace_groups)
        ).collect_vec();

    (monitor_groups, unmanaged_workspaces)
}
