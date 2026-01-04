use std::process::Command;

/// Send a macOS notification via osascript
pub fn notify(title: &str, message: &str, sound: Option<&str>) {
	let script = if let Some(sound_name) = sound {
		format!(
			r#"display notification "{}" with title "{}" sound name "{}""#,
			escape_applescript(message),
			escape_applescript(title),
			sound_name
		)
	} else {
		format!(
			r#"display notification "{}" with title "{}""#,
			escape_applescript(message),
			escape_applescript(title)
		)
	};

	let _ = Command::new("osascript").arg("-e").arg(&script).output();
}

fn escape_applescript(s: &str) -> String {
	s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Notify that an agent needs input
pub fn notify_needs_input(agent_name: &str, sound: &str) {
	notify("swarm", &format!("{} needs input", agent_name), Some(sound));
}

/// Notify that an agent finished
pub fn notify_done(agent_name: &str, sound: &str) {
	notify("swarm", &format!("{} completed", agent_name), Some(sound));
}

/// Notify of an error
#[allow(dead_code)]
pub fn notify_error(agent_name: &str, message: &str, sound: &str) {
	notify(
		"swarm",
		&format!("{}: {}", agent_name, message),
		Some(sound),
	);
}
