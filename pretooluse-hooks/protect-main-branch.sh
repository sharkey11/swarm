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

# Check if we're in a jj repo (colocated)
is_jj_repo() {
  [[ -d ".jj" ]] || [[ -d "$(git rev-parse --show-toplevel 2>/dev/null)/.jj" ]]
}

# For jj repos: extract the bookmark from jj git push command
# jj git push -b sharkey11/feature -> returns "sharkey11/feature"
get_jj_push_bookmark() {
  local cmd="$1"
  # Match -b or --bookmark followed by the bookmark name
  if [[ "$cmd" =~ jj\ git\ push.*-b[[:space:]]+([^[:space:]]+) ]]; then
    echo "${BASH_REMATCH[1]}"
  elif [[ "$cmd" =~ jj\ git\ push.*--bookmark[[:space:]]+([^[:space:]]+) ]]; then
    echo "${BASH_REMATCH[1]}"
  else
    echo ""
  fi
}

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

# Check if command is jj git push - handle jj repos specially
if [[ "$command" =~ jj\ git\ push ]]; then
  bookmark=$(get_jj_push_bookmark "$command")
  if [[ "$bookmark" == "main" || "$bookmark" == "master" ]]; then
    echo "BLOCKED: Cannot push main/master bookmark" >&2
    exit 2
  fi
  # jj git push with a non-main bookmark is safe
  exit 0
fi

# Check if command is git push - block ANY push that targets main/master
if [[ "$command" =~ git\ push ]] || [[ "$command" =~ git\ -C\ .+\ push ]]; then
  # In jj colocated repos, git HEAD points to main but jj working copy may be different
  # Allow pushes that explicitly target a non-main ref (e.g., HEAD:refs/heads/feature)
  if is_jj_repo; then
    # Allow: git push origin HEAD:refs/heads/sharkey11/feature
    if [[ "$command" =~ HEAD:refs/heads/([^[:space:]]+) ]]; then
      target_branch="${BASH_REMATCH[1]}"
      if [[ "$target_branch" != "main" && "$target_branch" != "master" ]]; then
        exit 0
      fi
    fi
  fi

  # Block if pushing while on main (non-jj repos or implicit push)
  current_branch=$(get_branch_for_command "$command")
  if [[ "$current_branch" == "main" || "$current_branch" == "master" ]]; then
    # In jj repos, suggest using jj git push instead
    if is_jj_repo; then
      echo "BLOCKED: In jj repo, use 'jj git push --bookmark <name>' instead" >&2
    else
      echo "BLOCKED: Cannot push from $current_branch branch" >&2
    fi
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
