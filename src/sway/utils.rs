use swayipc::{Connection, Output};

// TODO move some of the helpers tha don't edit state from ./commands.rs to here and rename all these files


/// Get list of active outputs from the sway IPC connection.
/// 
/// # Panics
/// Panics if the request fails
pub fn get_active_monitors(connection: &mut Connection) -> Vec<Output> {
    let outputs = connection.get_outputs().expect("Failed to get outputs");
    outputs.iter()
        .filter(|output| output.active)
        .cloned()
        .collect()
}

/// Concatenate output make, model, and serial number into a descriptor.
/// This format is recognized by `sway-output`.
pub fn get_output_descriptor(output: &Output) -> String {
    format!(
        "{} {} {}",
        &output.make,
        &output.model,
        if output.serial.is_empty() { "Unknown" } else { &output.serial },
    )
}

/// Find ouptut by its name and return its descriptor.
/// 
/// # Panics
/// Panics if requesting list of ouputs from Sway IPC fails
pub fn get_output_descriptor_by_name(connection: &mut Connection, name: &str) -> Option<String> {
    get_active_monitors(connection).iter()
        .find(|output| &output.name == name)
        .map(get_output_descriptor)
}

/// Find ouptut by its descriptor and return its name.
/// 
/// # Panics
/// Panics if requesting list of ouputs from Sway IPC fails
pub fn get_output_name_by_descriptor(connection: &mut Connection, descriptor: &str) -> Option<String> {
    get_active_monitors(connection).iter()
        .find(|output| get_output_descriptor(output) == descriptor)
        .map(|output| output.name.clone())
}
