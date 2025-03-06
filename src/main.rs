use clap::Parser;
use swayipc::Connection;

mod workspaces;
mod config;
mod cli;
mod sway_commands;

use config::Config;
use cli::{Cli, Subcommands};



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
        Subcommands::Switch { destination, mon_group } => {
            workspaces::switch_workspace_groups(
                &mut connection,
                &config,
                &mon_group.unwrap_or(config.groups[0].get_name().to_string()),
                &destination,
            );
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
                workspaces::print_state_text
            };
            if subscribe {
                workspaces::subscribe_and_print(connection, printer);
            } else {
                printer(&mut connection);
            }
        },
    };
}
