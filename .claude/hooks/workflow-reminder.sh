#!/bin/bash
# Hook: UserPromptSubmit
# Purpose: Inject workflow reminders when implementation tasks are detected
# Output to stdout becomes context for Claude

# Read the prompt from stdin
PROMPT=$(cat)

# Check if this looks like an implementation task
if echo "$PROMPT" | grep -qiE "(implement|build|create|add|fix|update|write|develop|make).*\b(feature|function|component|module|aggregate|command|endpoint|api|handler|service)\b"; then
    cat << 'EOF'
WORKFLOW REMINDER: This looks like an implementation task.

Before writing code, ensure you have:
1. Classified the work with /classify-work
2. Verified ceremonies with /check-ceremonies
3. Created any missing artifacts (Example Map, BDD scenarios)

When done, run /check-done before claiming completion.
EOF
fi

# Always exit 0 to not block
exit 0
