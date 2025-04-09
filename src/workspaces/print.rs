use itertools::{Either, Itertools};
use swayipc::{Connection, EventType};

use crate::sway::commands::get_workspaces;

use super::workspace_id::WorkspaceId;


pub fn get_current_index(connection: &mut Connection, monitor_group: &str) -> String {
    let workspaces = get_workspaces(connection);
    let visible_workspace = workspaces.iter()
        .filter(|workspace| workspace.visible)
        .filter_map(|workspace| WorkspaceId::parse_safe(&workspace.name))
        .filter(|workspace_id| workspace_id.get_monitor_group_name() == monitor_group)
        .next();

    visible_workspace
        .map_or("?".to_string(), |workspace_id| workspace_id.get_index().to_string())
}

pub fn subscribe_and_print(connection: &mut Connection, printer: fn(&mut Connection)) {
    // Print initial state
    printer(connection);
    // Open a new connection to listen for events
    let listen_connection = Connection::new().expect("Failed to connect to swayipc");
    let events = listen_connection.subscribe([EventType::Workspace]).expect("Failed to subscribe to workspace events");
    // Print updates as events come
    for _ in events {
        printer(connection);
    }
}

pub fn print_state_plain(connection: &mut Connection) {
    let simple_formatters = StateFormatters {
        unmanaged: StateFormattersUnmanaged {
            focused: |name| format!("*{}*", name),
            unfocused: |name| name.to_string(),
            separator: ", ".to_string(),
            whole: |workspaces| format!("Unmanaged: {}", workspaces),
        },
        managed: StateFormattersManaged {
            workspaces: StateFormattersManagedWorkspaces {
                focused: |index| format!("*{}*", index),
                unfocused: |index| index.to_string(),
                separator: ", ".to_string(),
            },
            group_name: StateFormattersManagedGroupName {
                focused: |name| format!("*{}*", name),
                unfocused: |name| name.to_string(),
            },
            mon_group: StateFormattersManagedMonGroup {
                whole: |mon_group, workspace_groups| format!("{}: {}", mon_group, workspace_groups),
                separator: " | ".to_string(),
            },
            whole: |groups| groups.to_string(),
        },
        separator: " | ".to_string(),
    };

    println!("{}", format_state(get_state(connection), &simple_formatters));
}

/// What a waybar module expects to see
#[derive(serde::Serialize)]
struct ModuleInput {
    text: String,
    tooltip: Option<String>,
    class: Option<String>,
}

pub fn print_waybar_module(connection: &mut Connection) {
    let display_text = get_state_rich_text(connection);
    let output = ModuleInput {
        text: display_text,
        class: Some("mumowrk".to_string()),
        tooltip: Some("You can switch between these using `mumowrk`".to_string()),
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}

/// Return a fromatted string with Pango markup representing the current state
pub fn get_state_rich_text(connection: &mut Connection) -> String {
    let formatters = StateFormatters {
        unmanaged: StateFormattersUnmanaged {
            focused: |name| format!("<u><b>{}</b></u>", name),
            unfocused: |name| name.to_string(),
            separator: ", ".to_string(),
            whole: |workspaces| format!("({}) | ", workspaces),
        },
        managed: StateFormattersManaged {
            workspaces: StateFormattersManagedWorkspaces {
                focused: |index| format!("<u><b>{}</b></u>", index),
                unfocused: |index| index.to_string(),
                separator: ", ".to_string(),
            },
            group_name: StateFormattersManagedGroupName {
                focused: |name| format!("<u>{}</u>", name),
                unfocused: |name| name.to_string(),
            },
            mon_group: StateFormattersManagedMonGroup {
                whole: |mon_group, workspace_groups| format!("{}: {}", mon_group, workspace_groups),
                separator: " | ".to_string(),
            },
            whole: |groups| format!("[ {} ]", groups),
        },
        separator: "".to_string(),
    };

    format_state(get_state(connection), &formatters)
}


type WorkspacesState = (Vec<(String, Vec<(i32, bool)>, bool)>, Vec<(String, bool)>);

#[derive(Debug)]
struct StateFormatters {
    unmanaged: StateFormattersUnmanaged,
    managed: StateFormattersManaged,
    separator: String,
}
#[derive(Debug)]
struct StateFormattersUnmanaged {
    focused: fn(&str) -> String,
    unfocused: fn(&str) -> String,
    separator: String,
    whole: fn(&str) -> String,
}
#[derive(Debug)]
struct StateFormattersManaged {
    workspaces: StateFormattersManagedWorkspaces,
    group_name: StateFormattersManagedGroupName,
    mon_group: StateFormattersManagedMonGroup,
    whole: fn(&str) -> String,
}
#[derive(Debug)]
struct StateFormattersManagedWorkspaces {
    focused: fn(i32) -> String,
    unfocused: fn(i32) -> String,
    separator: String,
}
#[derive(Debug)]
struct StateFormattersManagedGroupName {
    focused: fn(&str) -> String,
    unfocused: fn(&str) -> String,
}
#[derive(Debug)]
struct StateFormattersManagedMonGroup {
    whole: fn(&str, &str) -> String,
    separator: String,
}


fn format_state(
    (managed, unmanaged): WorkspacesState,
    formatters: &StateFormatters,
) -> String {
    let unmngd_fmts = &formatters.unmanaged;
    let mngd_fmts = &formatters.managed;
    let unmanaged_workspaces_str = unmanaged.iter()
        .map(|(name, focused)|
            (if *focused { unmngd_fmts.focused } else { unmngd_fmts.unfocused })(name)
        ).join(&unmngd_fmts.separator);

    let managed_groups_str = managed.iter()
        .map(|(mon_group, workspace_groups, focused)| {
            let workspace_groups_str = workspace_groups.iter()
                .map(|(index, focused)|
                    (if *focused { mngd_fmts.workspaces.focused } else { mngd_fmts.workspaces.unfocused })(*index)
                ).join(&mngd_fmts.workspaces.separator);
            let mon_group_label = (if *focused { mngd_fmts.group_name.focused } else { mngd_fmts.group_name.unfocused })(mon_group);
            (mngd_fmts.mon_group.whole)(&mon_group_label, &workspace_groups_str)
        })
        .join(&mngd_fmts.mon_group.separator);

    let output_text = (
        if unmanaged_workspaces_str.is_empty() { "".to_string() }
        else { (unmngd_fmts.whole)(&unmanaged_workspaces_str) + &formatters.separator }
    ) + &(mngd_fmts.whole)(&managed_groups_str);

    output_text
}

/// Get current workspaces and process the state into workspace groups state
fn get_state(connection: &mut Connection) -> WorkspacesState {
    let workspaces = get_workspaces(connection);
    
    let (unmanaged_workspaces, managed_ids): (Vec<_>, Vec<_>) =
        workspaces.iter().partition_map(|workspace| {
            match WorkspaceId::parse_safe(&workspace.name) {
                Some(id) => Either::Right((id, workspace.visible)),
                None => Either::Left((workspace.name.clone(), workspace.focused)),
            }
        });

    let focused_workspace_id = workspaces.iter()
        .find(|workspace| workspace.focused)
        .and_then(|workspace| WorkspaceId::parse_safe(&workspace.name));
    let workspaces_by_monitor_groups =
        managed_ids.iter().into_group_map_by(|(id, _)| id.get_monitor_group_name());
    let monitor_groups = workspaces_by_monitor_groups.iter()
        .map(|(group, workspaces)| {
            let workspace_groups = workspaces.iter()
                .unique_by(|(id, _)| id.get_index())
                .sorted_by_key(|(id, _)| id.get_index())
                .map(|(id, visible)| (id.get_index(), *visible))
                .collect();
            (*group, workspace_groups)
        })
        .sorted_by_key(|(group, _)| *group)
        .map(|(group, workspace_groups)| {
            let group_is_focused = focused_workspace_id.clone().map_or(
                false,
                |focused_id| focused_id.get_monitor_group_name() == group,
            );
            (group.to_string(), workspace_groups, group_is_focused)
        }
        ).collect_vec();

    (monitor_groups, unmanaged_workspaces)
}
