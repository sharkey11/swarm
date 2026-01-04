# Move to Git Worktree

Move this session to an isolated git worktree while preserving all current changes.

## When to Use

- When you realize mid-task you need branch isolation
- When working on a task that might conflict with other parallel agents
- When you want a clean git state based on latest main

## Process

1. **Stash current changes** (if any uncommitted work exists)
2. **Fetch latest main** from origin
3. **Create worktree** with a new branch based on origin/main
4. **CD into worktree**
5. **Apply stashed changes** to continue where you left off

## Steps

### Step 1: Check for uncommitted changes and stash them

```bash
# Check if there are changes to preserve
git status --porcelain
```

If there are changes, stash them with a descriptive message:

```bash
git stash push -m "worktree-migration-$(date +%Y%m%d-%H%M%S)" --include-untracked
```

### Step 2: Get the worktree details

Ask the user or derive from context:
- **Branch name**: Use format `{git_username}/{task-slug}` (e.g., `sharkey11/fix-auth-bug`)
- **Worktree path**: Use `~/worktrees/{task-slug}`

### Step 3: Fetch and create worktree

```bash
# Fetch latest main
git fetch origin main

# Create worktree with new branch from origin/main
git worktree add ~/worktrees/{task-slug} -b {branch-name} origin/main
```

### Step 4: Move to worktree

```bash
cd ~/worktrees/{task-slug}
```

### Step 5: Apply stashed changes

If changes were stashed in Step 1:

```bash
# List stashes to find ours
git stash list

# Apply the most recent stash (our worktree-migration stash)
git stash pop
```

### Step 6: Continue working

You're now in an isolated worktree with:
- Fresh codebase from origin/main
- Your uncommitted changes preserved
- A new branch ready for commits

## Cleanup (when done with worktree)

```bash
# From the main repo (not the worktree):
git worktree remove ~/worktrees/{task-slug}

# If branch was pushed and merged, delete it:
git branch -d {branch-name}
```

## Notes

- Worktrees share the same .git directory, so commits are visible across all worktrees
- Each worktree has its own working directory and index (staging area)
- You may need to run install commands (pnpm install, etc.) in the new worktree
