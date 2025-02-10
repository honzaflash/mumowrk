use clap::Parser;
use swayipc::Connection;

mod workspaces;
mod config;
mod cli;

use config::Config;
use workspaces::{init_workspaces, move_container_to_workspace_group, print_state_text, print_waybar_module, subscribe_and_print, switch_workspace_groups};
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
            init_workspaces(&mut connection, &config);
        },
        Subcommands::Switch { destination, mon_group } => {
            switch_workspace_groups(
                &mut connection,
                &config,
                &mon_group.unwrap_or(config.groups[0].get_name().to_string()),
                &destination,
            );
        },
        Subcommands::Move { destination, focus, mon_group } => {
            move_container_to_workspace_group(
                &mut connection,
                &config,
                &destination,
                mon_group.as_ref(),
                focus,
            );
        },
        Subcommands::Print { waybar_module, subscribe } => {
            let printer = if waybar_module {
                print_waybar_module
            } else {
                print_state_text
            };
            if subscribe {
                subscribe_and_print(connection, printer);
            } else {
                printer(&mut connection);
            }
        },
    };
}
