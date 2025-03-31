use clap::Parser;
use sway_commands::get_active_monitors;
use swayipc::Connection;

mod workspaces;
mod config;
mod cli;
mod sway_commands;
mod notify;

use config::Config;
use cli::{Cli, NotificationVerbosity, Subcommands};
use notify::dbus_notify;



fn main() {
    let args = Cli::parse();

    // Set the SWAYSOCK env var for this process to the option if provided
    // Connection::new() will read this env var to get the socket path
    args.socket.map(|socket| {
        std::env::set_var("SWAYSOCK", socket);
    });
    // Connect to the Sway IPC
    let mut connection = match Connection::new() {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("Failed to connect to swayipc: {}", error);
            return;
        }
    };

    let config = Config::load(&args.config);
    if config.groups.len() == 0 {
        eprintln!("No monitor groups configured!");
        return;
    }

    match args.command {
        Subcommands::Init => {
            workspaces::init_workspaces(&mut connection, &config);
        },
        Subcommands::Switch { destination, mon_group, notify } => {
            let target_mon_group = mon_group.unwrap_or(config.groups[0].get_name().to_string());
            workspaces::switch_workspace_groups(
                &mut connection,
                &config,
                &target_mon_group,
                &destination,
            );
            // TODO: move this to a function and add it to other subcommands
            let notification_text = match notify {
                NotificationVerbosity::None =>
                    { return; },
                NotificationVerbosity::Index =>
                    format!("<u><b>{}</b></u>", workspaces::get_current_index(&mut connection, &target_mon_group)),
                NotificationVerbosity::Summary =>
                    workspaces::get_state_rich_text(&mut connection),
            };
            let active_monitors = get_active_monitors(&mut connection);
            let monitor_group = config.get_group(&target_mon_group).unwrap();
            let target_monitor = &monitor_group.monitors[
                monitor_group.get_main_monitor_index(&active_monitors)
            ];
            dbus_notify(&notification_text, &target_monitor);
        },
        Subcommands::MoveGroup { from, to, mon_group } => {
            workspaces::swap_workspace_groups(
                &mut connection,
                &config,
                from,
                &to,
                mon_group.as_ref(),
            );
        },
        Subcommands::MoveContainer { destination, focus, mon_group } => {
            workspaces::move_container_to_workspace_group(
                &mut connection,
                &config,
                &destination,
                mon_group.as_ref(),
                focus,
            );
        },
        Subcommands::Reorganize {  } => {
            workspaces::reorganize_everything(&mut connection, &config);
        }
        Subcommands::Print { waybar_module, subscribe } => {
            let printer = if waybar_module {
                workspaces::print_waybar_module
            } else {
                workspaces::print_state_plain
            };
            if subscribe {
                workspaces::subscribe_and_print(&mut connection, printer);
            } else {
                printer(&mut connection);
            }
        },
    };
}
