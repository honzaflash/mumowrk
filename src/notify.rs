use dbus::arg::{self, Variant};
use dbus::Message;
use dbus::blocking::{BlockingSender, Connection as DbusConnection};
use std::time::Duration;
use swayipc::Connection as SwayConnection;

use crate::cli::NotificationVerbosity;
use crate::config::Config;
use crate::sway::commands::get_active_monitor_names;
use crate::sway::utils::get_output_name_by_descriptor;
use crate::workspaces;

pub fn maybe_send_update_notification(
    connection: &mut SwayConnection,
    notify: NotificationVerbosity,
    config: &Config,
    target_mon_group: &str,
) {
    let notification_text = match notify {
        NotificationVerbosity::None =>
            { return; },
        NotificationVerbosity::Index =>
            format!("<u><b>{}</b></u>", workspaces::get_current_index(connection, &target_mon_group)),
        NotificationVerbosity::Summary =>
            workspaces::get_state_rich_text(connection),
    };
    let active_monitors = get_active_monitor_names(connection);
    let monitor_group = config.get_group(&target_mon_group).unwrap();
    let target_monitor = monitor_group.monitors[
        monitor_group.get_main_monitor_index(&active_monitors)
    ].clone();
    // We need to translate to output name if monitor is configured using its descriptor
    let output_name = get_output_name_by_descriptor(connection, &target_monitor).unwrap_or(target_monitor);

    dbus_notify(&notification_text, &output_name);
}

pub(crate) fn dbus_notify(text: &str, target_monitor: &str) {
    let connection = DbusConnection::new_session().expect("Failed to connect to D-Bus");

    // https://specifications.freedesktop.org/notification-spec/latest/protocol.html#command-notify
    let msg = Message::new_method_call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify"
    ).expect("Failed to create new D-Bus method call message")
        // appname
        .append1("mumowrk")
        // notification to update
        .append1(0u32)
        // icon
        .append1("")
        // summary
        .append1(&format!("mon:{}", target_monitor))
        // body
        .append1(text)
        // actions (none)
        .append1(Vec::<String>::new())
        // hints
        .append1(arg::PropMap::from([
            ("urgency".to_string(), Variant(Box::new(0u8) as Box<dyn arg::RefArg>)),
            ("category".to_string(), Variant(Box::new("pop-up".to_string()) as Box<dyn arg::RefArg>)),
            ("transient".to_string(), Variant(Box::new(true) as Box<dyn arg::RefArg>)),
        ]))
        // timeout (-1 -> let notification server decide)
        .append1(-1i32);

    connection.send_with_reply_and_block(msg, Duration::from_millis(5000))
        .expect("Failed to send D-Bus message");
}
