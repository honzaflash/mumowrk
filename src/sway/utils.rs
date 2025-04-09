use swayipc::Output;

// TODO move some of the helpers tha don't edit state from ./commands.rs to here and rename all these files

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
