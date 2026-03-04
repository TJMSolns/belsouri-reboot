#!/bin/bash
# Hook: PreToolUse (Edit on domain files)
# Purpose: Check if ceremonies exist before allowing edits to domain code
# Injects findings to stdout, blocks (exit 2) if critical artifacts missing

# Read the tool input from stdin
INPUT=$(cat)

# Extract file path from Edit tool input
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

# If no file path, allow
if [ -z "$FILE_PATH" ]; then
    exit 0
fi

PROJECT_ROOT="/home/tjm/Cloud/GitHub/belsouri"

# Check if this is a domain file (aggregates, events, commands)
if echo "$FILE_PATH" | grep -qE "src-tauri/src/modules/|src-tauri/src/commands/|src-tauri/src/projections/"; then

    # Extract context name from path
    if echo "$FILE_PATH" | grep -q "modules/scheduling"; then
        CONTEXT="scheduling"
    elif echo "$FILE_PATH" | grep -q "modules/platform"; then
        CONTEXT="platform"
    elif echo "$FILE_PATH" | grep -q "modules/outreach"; then
        CONTEXT="outreach"
    else
        # Unknown context - allow but warn
        echo "NOTE: Editing domain file in unrecognized context. Ensure ceremonies are complete."
        exit 0
    fi

    # Check for event storming doc
    EVENT_STORMING="$PROJECT_ROOT/doc/internal/domain/event-storming/${CONTEXT}-events.md"
    if [ ! -f "$EVENT_STORMING" ]; then
        echo "WARNING: No event storming doc found for $CONTEXT context."
        echo "Expected: $EVENT_STORMING"
        echo "Consider running /classify-work to determine if ceremonies are needed."
    fi

    # Check for aggregate docs
    AGGREGATES_DIR="$PROJECT_ROOT/doc/internal/domain-models/aggregates"
    AGGREGATE_COUNT=$(ls -1 "$AGGREGATES_DIR"/*.md 2>/dev/null | wc -l)
    if [ "$AGGREGATE_COUNT" -eq 0 ]; then
        echo "WARNING: No aggregate docs found in $AGGREGATES_DIR"
    fi
fi

# Check if this is a feature file
if echo "$FILE_PATH" | grep -qE "features/.*\.feature$"; then
    FEATURE_NAME=$(basename "$FILE_PATH" .feature)
    EXAMPLE_MAP="$PROJECT_ROOT/doc/internal/scenarios/example-maps/${FEATURE_NAME}-examples.md"

    if [ ! -f "$EXAMPLE_MAP" ]; then
        echo "WARNING: No Example Map found for this feature."
        echo "Expected: $EXAMPLE_MAP"
        echo "Per HOW-WE-WORK.md, Example Mapping should precede BDD scenarios."
    fi
fi

# Always allow (warnings only, not blocks)
exit 0
