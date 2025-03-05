use clap::{Parser, Subcommand};

const DEFAULT_CONFIG_PATH: &str = "$HOME/.config/mumowrk/config.yml";

/// Multi Monitor Workspace Manager
#[derive(Parser, Debug)]
#[command(version, about, long_about = "MuMoWrk is a tool that manages workspaces across multiple monitors.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Subcommands,

    /// Path to config file
    #[arg(short, long, value_name = "CONFIG_PATH", default_value = DEFAULT_CONFIG_PATH, required = false)]
    pub config: String,

    /// Path to the IPC socket
    #[arg(short, long, value_name = "SOCKET_PATH", required = false)]
    pub socket: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Initialize workspaces based on the configuration
    Init,
    /// Switch between workspace groups
    Switch {
        /// Absolute index or relative increment ([+-]N) for destination workspace group
        #[arg(value_name = "DESTINATION", allow_hyphen_values = true)]
        destination: String,
        /// Target monitor group name (default: first group in config)
        #[arg(short, long, value_name = "MONITOR_GROUP", required = false)]
        mon_group: Option<String>,
    },
    /// Move (reorder) workspace groups
    #[clap(visible_alias = "swap")]
    MoveGroup {
        // TODO: change `from` and `to` to `A` and `B` or something
        //       Also rename the command to SwapGroup probably
        /// Absolute index of target workspace group
        #[arg(value_name = "FROM")]
        from: i32,
        // TODO: validate
        /// Absolute index or relative increment ([+-]N) for destination
        #[arg(value_name = "TO", allow_hyphen_values = true)]
        to: String,
        /// Target monitor group name (default: focused group)
        #[arg(short, long, value_name = "MONITOR_GROUP", required = false)]
        mon_group: Option<String>,
    },
    /// Move containers between workspace groups
    #[clap(visible_alias = "move")]
    MoveContainer {
        /// Absolute index or relative increment ([+-]N) for destination workspace group
        #[arg(value_name = "DESTINATION", allow_hyphen_values = true)]
        destination: String,
        /// Change focus to targer workspace group after moving
        #[arg(short, long)]
        focus: bool,
        /// Target monitor group name (default: focused group)
        #[arg(short, long, value_name = "MONITOR_GROUP", required = false)]
        mon_group: Option<String>,
    },
    /// Print the current state of workspaces
    Print {
        /// Print state as JSON input for a waybar module
        #[arg(long, short)]
        waybar_module: bool,
        /// Subscribe to IPC events and keep printing state updates on change
        #[arg(long, short)]
        subscribe: bool,
    },
}
