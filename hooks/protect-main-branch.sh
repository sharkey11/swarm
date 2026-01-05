#!/bin/bash
# Prevents commits and pushes to main branch

# Read the tool input from stdin (Claude passes JSON)
input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command // empty' 2>/dev/null)

# If we couldn't parse, let it through
if [[ -z "$command" ]]; then
  exit 0
fi

# Whitelist: Allow main branch operations for test repos
remote_url=$(git remote get-url origin 2>/dev/null || echo "")
if [[ "$remote_url" =~ whopio/pr-dm-notifications-test ]]; then
  exit 0
fi
# Also check if command is cd'ing into the test repo directory
if [[ "$command" =~ pr-dm-notifications-test ]]; then
  exit 0
fi

# Helper function to get branch, handling cd commands
get_branch_for_command() {
  local cmd="$1"
  local target_dir=""

  # Check if command starts with cd to a directory
  if [[ "$cmd" =~ ^cd[[:space:]]+([^&\;]+) ]]; then
    target_dir="${BASH_REMATCH[1]}"
    # Trim whitespace
    target_dir="${target_dir%% }"
  fi

  if [[ -n "$target_dir" && -d "$target_dir" ]]; then
    git -C "$target_dir" rev-parse --abbrev-ref HEAD 2>/dev/null || echo ""
  else
    git rev-parse --abbrev-ref HEAD 2>/dev/null || echo ""
  fi
}

# Check if command is git commit
if [[ "$command" =~ git\ commit ]] || [[ "$command" =~ git\ -C\ .+\ commit ]]; then
  current_branch=$(get_branch_for_command "$command")
  if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then
    echo "BLOCKED: Cannot commit directly to $current_branch branch" >&2
    echo "Create a feature branch first: git checkout -b sharkey11/your-feature" >&2
    exit 2
  fi
fi

# Check if command is git push - block ANY push that targets main/master
if [[ "$command" =~ git\ push ]] || [[ "$command" =~ git\ -C\ .+\ push ]]; then
  # Block if pushing while on main
  current_branch=$(get_branch_for_command "$command")
  if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then
    echo "BLOCKED: Cannot push from $current_branch branch" >&2
    exit 2
  fi

  # Block if command explicitly pushes to main/master as a ref target
  # Match patterns like: "push origin main", "push origin master", "push main", "push master"
  # But NOT: "push -u origin sharkey11/protect-main-ci" (main in branch name is fine)
  if [[ "$command" =~ push[[:space:]]+(--[^[:space:]]+[[:space:]]+)*([^[:space:]]+[[:space:]]+)?(main|master)([[:space:]]|$) ]]; then
    echo "BLOCKED: Cannot push to main/master branch" >&2
    echo "Push to your feature branch instead" >&2
    exit 2
  fi
fi

exit 0
