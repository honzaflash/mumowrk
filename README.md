# MuMoWrk aka `mmwrk`
The MultiMonitor Workspace manager for Sway (could also work for i3).

This readme is still WIP

## Features
- workspaces that span multiple monitors
- create multiple monitor groups each with their own set of workspaces
  - this way you can keep your comms monitor and switch only between
    different work on the rest of the monitors for instance
- switch to a specific workspace or to one relative to the current workspace
- print status compatible with waybar modules

Intended for use with keybindings.

# Install
Run `cargo build --release` which will create an executable
at `mumowrk/target/release/mmwrk`.

Then you can copy it or link to it somewhere on your `PATH`
like `/usr/local/bin/mmwrk`.

Note that `PATH` in the shell used by `sway` might be different
from the `PATH` in your prefered daily-use shell.

# Usage
To see the basic usage run `mmwrk help`, for subcommands
`mmwrk switch --help`, etc.

First you need a config file that will tell `mmwrk`
which monitors should be in a group. This should be here:
`~/.config/mumowrk/config.yml`. You can look at the reference
configuration `config.example.yml` in the project directory.

To initialize workspaces run `mmwrk init` then switch between them
with `mmwrk switch INDEX -m GROUP`.

You might want to add something like this to your sway config:
```
exec mmwrk init

bindsym $mod+Ctrl+$left exec mmwrk switch -m G1 -1
bindsym $mod+Ctrl+Left exec  mmwrk switch -m G1 -1
bindsym $mod+Ctrl+$right exec mmwrk switch -m G1 +1
bindsym $mod+Ctrl+Right exec  mmwrk switch -m G1 +1
```

