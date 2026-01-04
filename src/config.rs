// TODO (future features):
// - Support multiple agent types: Opencode, Codex, custom agents
// - Add agent-specific settings (API keys, models, etc.)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG: &str = r#"
[general]
default_agent = "claude"
worktree_dir = "~/worktrees"
poll_interval_ms = 1000
logs_dir = "~/.swarm/logs"
tasks_dir = "~/.swarm/tasks"
daily_dir = "~/.swarm/daily"

[notifications]
enabled = true
sound_needs_input = "Ping"
sound_done = "Glass"
sound_error = "Basso"

[keybindings]
prefix = "ctrl-a"

# Bash commands that run without permission prompts in Claude Code.
# Format: "Bash(command:*)" - the :* allows any arguments
[allowed_tools]
tools = [
  # Navigation & filesystem (read-only)
  "Bash(cd:*)",
  "Bash(ls:*)",
  "Bash(pwd:*)",
  "Bash(cat:*)",
  "Bash(head:*)",
  "Bash(tail:*)",
  "Bash(less:*)",
  "Bash(file:*)",
  "Bash(find:*)",
  "Bash(which:*)",
  "Bash(type:*)",
  "Bash(wc:*)",
  "Bash(du:*)",
  "Bash(df:*)",
  "Bash(tree:*)",
  # Git (read-only)
  "Bash(git status:*)",
  "Bash(git log:*)",
  "Bash(git diff:*)",
  "Bash(git show:*)",
  "Bash(git branch:*)",
  "Bash(git remote:*)",
  "Bash(git stash list:*)",
  "Bash(git rev-parse:*)",
  "Bash(git describe:*)",
  "Bash(git config --get:*)",
  "Bash(git ls-files:*)",
  "Bash(git ls-tree:*)",
  # GitHub CLI (read-only)
  "Bash(gh pr view:*)",
  "Bash(gh pr list:*)",
  "Bash(gh pr diff:*)",
  "Bash(gh pr checks:*)",
  "Bash(gh issue view:*)",
  "Bash(gh issue list:*)",
  "Bash(gh api:*)",
  "Bash(gh release list:*)",
  "Bash(gh release view:*)",
  # Package managers (read-only)
  "Bash(npm list:*)",
  "Bash(npm ls:*)",
  "Bash(npm view:*)",
  "Bash(pnpm list:*)",
  "Bash(pnpm ls:*)",
  "Bash(yarn list:*)",
  "Bash(cargo tree:*)",
  "Bash(cargo metadata:*)",
  # Build & test
  "Bash(cargo build:*)",
  "Bash(cargo check:*)",
  "Bash(cargo test:*)",
  "Bash(cargo clippy:*)",
  "Bash(cargo fmt --check:*)",
  "Bash(npm run:*)",
  "Bash(pnpm run:*)",
  "Bash(yarn run:*)",
  "Bash(make:*)",
  "Bash(go build:*)",
  "Bash(go test:*)",
  # Docker (read-only)
  "Bash(docker ps:*)",
  "Bash(docker images:*)",
  "Bash(docker logs:*)",
  # Misc safe
  "Bash(echo:*)",
  "Bash(date:*)",
  "Bash(env:*)",
  "Bash(printenv:*)",
  "Bash(grep:*)",
  "Bash(rg:*)",
  "Bash(ag:*)",
]
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub general: General,
	pub notifications: Notifications,
	pub keybindings: Keybindings,
	#[serde(default)]
	pub allowed_tools: AllowedTools,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
	pub default_agent: String,
	pub worktree_dir: String,
	pub poll_interval_ms: u64,
	pub logs_dir: String,
	#[serde(default = "default_daily_dir")]
	pub daily_dir: String,
	#[serde(default = "default_tasks_dir")]
	pub tasks_dir: String,
	#[serde(default = "default_branch_prefix")]
	pub branch_prefix: String,
	#[serde(default = "default_status_style")]
	pub status_style: String, // "emoji", "unicode", "text"
}

fn default_status_style() -> String {
	"emoji".to_string()
}

fn default_branch_prefix() -> String {
	"sharkey11/".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notifications {
	pub enabled: bool,
	pub sound_needs_input: String,
	pub sound_done: String,
	pub sound_error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
	pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllowedTools {
	#[serde(default)]
	pub tools: Vec<String>,
}

pub fn load_or_init() -> Result<Config> {
	let base_dir = base_dir()?;
	if !base_dir.exists() {
		fs::create_dir_all(&base_dir)?;
	}

	let agents_dir = base_dir.join("agents");
	let logs_dir = base_dir.join("logs");
	let daily_dir = base_dir.join("daily");
	let sessions_dir = base_dir.join("sessions");
	if !agents_dir.exists() {
		fs::create_dir_all(&agents_dir)?;
	}
	if !logs_dir.exists() {
		fs::create_dir_all(&logs_dir)?;
	}
	if !daily_dir.exists() {
		fs::create_dir_all(&daily_dir)?;
	}
	if !sessions_dir.exists() {
		fs::create_dir_all(&sessions_dir)?;
	}
	let tasks_dir_expanded = expand_path(&default_tasks_dir());
	let tasks_dir = Path::new(&tasks_dir_expanded);
	if !tasks_dir.exists() {
		let _ = fs::create_dir_all(tasks_dir);
	}

	let config_path = base_dir.join("config.toml");
	if !config_path.exists() {
		fs::write(&config_path, DEFAULT_CONFIG.trim_start())?;
	}
	let content = fs::read_to_string(&config_path)?;
	let mut cfg: Config = toml::from_str(&content)?;
	cfg.general.logs_dir = expand_path(&cfg.general.logs_dir);
	cfg.general.worktree_dir = expand_path(&cfg.general.worktree_dir);
	cfg.general.daily_dir = expand_path(&cfg.general.daily_dir);
	cfg.general.tasks_dir = expand_path(&cfg.general.tasks_dir);
	for path in [
		cfg.general.logs_dir.as_str(),
		cfg.general.daily_dir.as_str(),
		cfg.general.tasks_dir.as_str(),
	] {
		let _ = fs::create_dir_all(Path::new(path));
	}
	Ok(cfg)
}

pub fn expand_path(input: &str) -> String {
	if input.starts_with("~/") {
		if let Some(home) = dirs::home_dir() {
			return home
				.join(input.trim_start_matches("~/"))
				.to_string_lossy()
				.into_owned();
		}
	}
	input.to_string()
}

fn default_daily_dir() -> String {
	"~/.swarm/daily".to_string()
}

fn default_tasks_dir() -> String {
	"~/.swarm/tasks".to_string()
}

pub fn base_dir() -> Result<PathBuf> {
	dirs::home_dir()
		.map(|p| p.join(".swarm"))
		.ok_or_else(|| anyhow::anyhow!("Failed to resolve home directory"))
}

pub fn session_store_dir() -> Result<PathBuf> {
	let dir = base_dir()?.join("sessions");
	fs::create_dir_all(&dir)?;
	Ok(dir)
}

pub fn snapshots_dir() -> Result<PathBuf> {
	let dir = base_dir()?.join("snapshots");
	fs::create_dir_all(&dir)?;
	Ok(dir)
}

