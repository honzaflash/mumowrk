use itertools::{Either, Itertools};
use swayipc::Connection;

use super::workspace_id::WorkspaceId;

pub fn print_state(connection: &mut Connection) {
    let workspaces = connection.get_workspaces().expect("Failed to get workspaces");
    let (unmanaged_ids, managed_ids): (Vec<_>, Vec<_>) =
        workspaces.iter().partition_map(|workspace| {
            match WorkspaceId::parse_safe(&workspace.name) {
                Some(id) => Either::Right(id),
                None => Either::Left((workspace.name.clone(), workspace.focused)),
            }
        });

    let unmanaged_workspaces = unmanaged_ids.iter()
        .map(|(name, focused)| if *focused { format!("*{}*", name) } else { name.clone() } ) 
        .join(", ");

    let focused_workspace_id = workspaces.iter()
        .find(|workspace| workspace.focused)
        .map(|workspace| WorkspaceId::parse(&workspace.name));
    let workspaces_by_monitor_groups = managed_ids.iter().into_group_map_by(|id| id.get_monitor_group_name());
    let monitor_groups = workspaces_by_monitor_groups.iter()
        .map(|(group, workspaces)| {
            let workspace_groups = workspaces.iter()
                .unique_by(|id| id.get_index())
                .map(|id| {
                    let group_is_focused = focused_workspace_id.clone().map_or(
                        false,
                        |focused_id| focused_id.get_monitor_group_name() == *group && focused_id.get_index() == id.get_index(),
                    );
                    if group_is_focused { format!("*{}*", id.get_index()) } else { format!("{}", id.get_index()) }
                }).join(", ");
            (*group, workspace_groups)
        })
        .sorted_by_key(|(group, _)| *group)
        .map(|(group, workspace_groups)|
            format!("{}: {}", group, workspace_groups)
        )
        .join(" | ");

    let output = [
        if unmanaged_workspaces.is_empty() { "".to_string() } else { format!("Unmanaged: {} | ", unmanaged_workspaces) },
        monitor_groups,
    ].join("");

    println!("{}", output);
}
