#!/bin/bash
# Hook: PreToolUse (Bash with git commit)
# Purpose: Verify build passes before allowing commit
# Exit 2 to block, 0 to allow

# Read the tool input from stdin
INPUT=$(cat)

# Check if this is a git commit command
if echo "$INPUT" | jq -e '.tool_input.command' 2>/dev/null | grep -q "git commit"; then
    echo "Running pre-commit checks..." >&2

    # Change to project root
    cd /home/tjm/Cloud/GitHub/belsouri

    # Run cargo test
    if ! cd src-tauri && cargo test --quiet 2>&1; then
        echo "BLOCKED: cargo test failed. Fix tests before committing." >&2
        exit 2
    fi
    cd ..

    # Run pnpm check
    if ! pnpm check --quiet 2>&1; then
        echo "BLOCKED: pnpm check failed. Fix TypeScript errors before committing." >&2
        exit 2
    fi

    echo "Pre-commit checks passed." >&2
fi

exit 0
