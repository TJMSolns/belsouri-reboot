# Belsouri Custom Skills

These skills enforce the ceremony-based SDLC from HOW-WE-WORK.md.

## Available Skills

| Skill | Command | When to Use |
|-------|---------|-------------|
| work-classifier | `/classify-work` | FIRST - before any implementation task |
| ceremony-checker | `/check-ceremonies` | After classification, before coding |
| done-checker | `/check-done` | Before claiming any work is complete |

## Workflow

```
User: "Implement X"
    ↓
/classify-work X
    ↓
Classification: NEW_FEATURE in Scheduling context
Required: Phase 2 artifacts (Example Map, BDD Scenarios)
    ↓
/check-ceremonies scheduling
    ↓
Missing: features/booking-flow.feature
    ↓
(Create missing artifacts, get approval)
    ↓
/check-ceremonies scheduling
    ↓
All artifacts present - READY TO IMPLEMENT
    ↓
(Implementation with TDD)
    ↓
/check-done "booking flow implementation"
    ↓
PASS - ready to commit
```

## Hooks (Automatic)

| Hook | Event | Purpose |
|------|-------|---------|
| workflow-reminder.sh | UserPromptSubmit | Reminds to use skills for implementation tasks |
| pre-commit-checks.sh | PreToolUse (Bash) | Blocks commits if tests fail |

## Files

```
.claude/
├── settings.json          # Project settings + hook config
├── skills/
│   ├── README.md          # This file
│   ├── work-classifier/
│   │   └── SKILL.md       # /classify-work
│   ├── ceremony-checker/
│   │   └── SKILL.md       # /check-ceremonies
│   └── done-checker/
│       └── SKILL.md       # /check-done
└── hooks/
    ├── workflow-reminder.sh    # Injects workflow reminders
    └── pre-commit-checks.sh    # Pre-commit quality gate
```
