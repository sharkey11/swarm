# AGENTS.md - Swarm Service

Terminal dashboard for managing multiple AI coding agents in parallel.

## Quick Commands

```bash
# Build
cargo build --package swarm

# Run
cargo run --package swarm

# Build release
cargo build --package swarm --release

# Check (fast compile check)
cargo check --package swarm

# Format
cargo fmt --package swarm

# Lint
cargo clippy --package swarm -- -D warnings
```

## Architecture

```
swarm/
├── src/
│   ├── main.rs        # TUI app, CLI parsing, all UI rendering
│   ├── config.rs      # Config loading (~/.swarm/config.toml)
│   ├── model.rs       # Data structures (AgentSession, TaskEntry, etc.)
│   ├── detection.rs   # Agent status detection (NeedsInput, Running, etc.)
│   ├── logs.rs        # Log file tailing, ANSI stripping
│   ├── notify.rs      # macOS notifications via osascript
│   └── tmux.rs        # tmux session management
└── Cargo.toml
```

## Key Files

### `main.rs` (~2000 lines)
The main TUI application. Contains:
- **CLI parsing** (clap) - `swarm`, `swarm new <name>`, `swarm status`
- **TUI rendering** (ratatui) - agents list, tasks list, preview panel
- **Event handling** - keyboard input, session polling
- **Session management** - create/kill tmux sessions

Key functions:
- `run_tui()` - main TUI loop
- `collect_sessions()` - discover swarm-* tmux sessions
- `load_tasks()` - load task files from tasks_dir
- `start_from_task()` / `start_from_task_yolo()` - create new agent session
- `create_task_and_start()` - "name your work" flow
- `is_prompt_line()` - detect if a line is asking for user input
- `clean_preview()` - clean up preview output for display

### `detection.rs`
Status detection logic. Detects agent state from tmux output:
- `NeedsInput` - prompt patterns like `[Y/n]`, `?`, `Should I`
- `Running` - recent output activity
- `Idle` - no output for 30s+
- `Done` - explicit `/swarm:done` marker or process exit

### `config.rs`
Config file parsing. Default config created at `~/.swarm/config.toml`.
Key settings:
- `general.tasks_dir` - where task files live
- `general.daily_dir` - daily log files
- `notifications.enabled` - macOS notifications
- `status_style` - unicode/emoji/text/minimal

### `model.rs`
Data structures:
- `AgentSession` - a running tmux session
- `AgentStatus` - NeedsInput/Running/Idle/Done/Unknown
- `TaskEntry` - a task file from tasks_dir
- `TaskInfo` - task metadata attached to a session

## Common Modifications

### Adding a new keybinding
1. Find the key handling section in `run_tui()` (search for `KeyCode::`)
2. Add the new key handler in the appropriate view (agents vs tasks)
3. Update the footer text in `agents_footer_text()` or `tasks_footer_text()`
4. Update the help modal content (search for `HELP MODAL`)

### Changing status detection patterns
Edit `detection.rs`:
- `needs_input_patterns` - regex patterns for detecting prompts
- Thresholds for running/idle detection

### Adding a new config option
1. Add field to struct in `config.rs`
2. Add default in `DEFAULT_CONFIG`
3. Use it in `main.rs`

### Modifying the preview panel
Search for `preview_lines_styled` in `main.rs`. The preview is built as `Vec<Line>` with styled spans.

### Modifying the agents list
Search for `let items: Vec<ListItem> = sessions` in `main.rs`.

### Modifying the tasks list
Search for `if showing_tasks {` in `main.rs`.

## Data Flow

1. **Session discovery**: `collect_sessions()` runs `tmux ls` to find `swarm-*` sessions
2. **Status detection**: For each session, read log file and detect status via patterns
3. **Polling**: Every `poll_interval_ms` (default 1000ms), refresh sessions and status
4. **Preview caching**: Preview output is cached and only refreshed on poll or selection change

## Dependencies

- `ratatui` - TUI framework
- `crossterm` - terminal backend
- `tokio` - async runtime (minimal use, mostly sync)
- `clap` - CLI argument parsing
- `chrono` - date/time handling
- `anyhow` - error handling
- `slug` - slugifying task names for session names

## Session/Task Relationship

- Sessions can optionally link to a task via `~/.swarm/sessions/<session>/task` file
- When viewing tasks, active sessions show a `●` indicator
- The "name your work" flow (`n` key) creates both a task file and a session

## Paths

- `~/.swarm/config.toml` - user config
- `~/.swarm/logs/` - session output logs (piped from tmux)
- `~/.swarm/sessions/` - per-session metadata
- `~/.swarm/tasks/` - task files (default, configurable)
- `~/.swarm/daily/` - daily logs (default, configurable)

## Hooks (Claude Commands)

Swarm includes Claude commands in `hooks/`:
- `done.md` - End session, log work
- `interview.md` - Detailed task planning
- `log.md` - Save progress to task file
- `poll-pr.md` - Monitor PR until CI green
- `qa-swarm.md` - QA testing for swarm

These are embedded in the binary via `include_str!()` and installed to `~/.claude/commands/` on first run.

## Testing

**IMPORTANT: When adding a new feature or flow, update `hooks/qa-swarm.md` with a test flow for it.**

Manual testing via `/qa-swarm` command. Key flows:
1. Launch with no sessions - should show hint
2. Create new agent (`n` key) - full "name your work" flow
3. Agent needing input - header should show count, preview should highlight prompt
4. Tasks view (`t` key) - should show `●` for tasks with active sessions
5. Attach/detach (`a` key, then Ctrl-b d)
6. Kill session (`d` key) - shows confirmation modal
7. First-run onboarding - set `hooks_installed = false` in config, run swarm

## Known Quirks

- Preview scrolling is anchored to bottom (latest output visible)
- YOLO mode (capital `Y`) adds `--dangerously-skip-permissions` flag
- Status detection can have false positives on `?` in code output
- Log files are tailed from end (last 64KB) for performance
