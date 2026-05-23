# Dayroll

![Dayroll wordmark](docs/assets/dayroll-wordmark-01-630x127.png)

A terminal-based task manager with a day-first workflow and nested task/calendar layout.

## Overview

Dayroll organizes work into a main **Tasks** panel, an **Overdue** subpanel, and a nested **Calendar** panel. Tasks can be assigned to any date, making it easy to see what needs attention and plan ahead.

Current layout:
- Main widget: Tasks + Overdue + Calendar
- Main widget top titles: left `日録 // DAYROLL`, center current date, right clock
- Bottom-left of main widget: search filter state chip (`FILTER idle|active`)
- Bottom-right of main widget: active theme chip (`theme:<name>`)
- Footer status bar: interaction hints and mode state

Title style:
- Main widget title uses a kanji + english lockup: `日録 // DAYROLL`

## Workflow

1. **Launch**: Dayroll opens on today's date, showing all overdue tasks and today's tasks
2. **Navigate**: Use arrow keys or j/k to move between tasks
3. **Add tasks**: Press `a` to add a new task
4. **Edit tasks**: Press `e` to edit a task's title, description, priority, or assigned date
5. **Move tasks**: Press `m` to move a task to a different date
6. **Complete tasks**: Press `Enter` or `Space` to mark tasks as done
7. **Delete tasks**: Press `d` to delete a task

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Left` / `Right` or `[` / `]` | Move between days |
| `{` / `}` or `H` / `L` | Move between months |
| `t` | Jump to today |

### Task Management
| Key | Action |
|-----|--------|
| `a` | Add new task |
| `e` | Edit task (title, priority, date) |
| `m` | Move task to different date |
| `d` | Delete selected task |
| `Enter` / `Space` | Toggle task done/pending |
| `u` | Undo last move/delete/toggle |
| `/` | Enter search mode |
| `l` / `h` | Expand/collapse selected task description |
| `T` / `Y` | Next/previous theme |
| `Esc` (in search) | Clear/exit search |
| `?` | Open/close help |
| `q` | Quit (with confirmation) |

### Search Behavior
- Press `/` to enter explicit search mode.
- Footer shows live search state: `[search: __] [Esc] clear` and current query as you type.
- While search mode is active, typed command letters are treated as search input (not task actions).
- Press `Esc` to clear the query and exit search mode.

## Panels

### Tasks
- Active list for non-overdue items on/around the selected day view
- Header includes right-aligned counters: `todo`, `done`, `overdue`
- Rows use selection marker + status dot + priority chip
- Done task labels render with strikethrough
- Tasks with description show tree glyphs (`▸` collapsed, `▾` expanded)
- Expanded description renders as nested child row (`└─ ...`)

### Overdue
- Incomplete tasks from past dates
- Rendered in its own subpanel beneath Tasks
- Intended to keep stale work visible without dominating the main queue

### Calendar
- Nested inside the same main widget as Tasks/Overdue
- Uses responsive split: side-by-side on wide terminals, stacked on narrow terminals
- Selected day now has a strong accent background for visibility

### Priority System
- **High**: `P1` chip
- **Medium**: `P2` chip
- **Low**: `P3` chip

### Status Markers
- `●` orange = todo (pending)
- `●` green = done
- `●` red = overdue

### Themes
- Built-in themes: `dayroll`, `nord`, `gruvbox`, `tokyo-night`
- Cycle with `T` (next) / `Y` (previous)
- Selected theme is persisted in `~/.config/dayroll/config.toml`

## Task Operations

### Adding a Task
1. Press `a` to open the add task dialog
2. Enter title in the `Title` field
3. Use `Tab` / `Shift+Tab` to switch fields
4. Use `←/→` to change priority and date picker keys for date
5. Press `Enter` to save

Quick-add tokens (in the title field):
- `!high` / `!medium` / `!low`
- `@today` / `@tomorrow` / `@YYYY-MM-DD`
- Example: `pay rent @tomorrow !high`

### Editing a Task
1. Select a task and press `e`
2. Edit title, description, priority, or date fields
3. Press Enter to save changes

### Moving a Task
1. Select a task and press `m`
2. Choose a new date from the calendar modal
3. Task is reassigned to the selected date

### Deleting a Task
1. Select a task and press `d`
2. Press `u` to undo if needed

## Storage

### Location
- Tasks: `~/.dayroll/todos.json`
- Runtime config (theme): `~/.config/dayroll/config.toml`

### Format
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Complete project report",
    "description": "Draft outline and send for review",
    "status": "pending",
    "priority": "high",
    "assigned_day": "2026-04-22",
    "completed_at": null
  }
]
```

**Fields:**
- `id`: UUID v4 unique identifier
- `title`: Task description
- `description`: Optional markdown notes/details
- `status`: "pending" or "done"
- `priority`: "low", "medium", or "high"
- `assigned_day`: Target date in ISO format (YYYY-MM-DD)
- `completed_at`: ISO timestamp when completed (null if pending)

## Roadmap

### In Progress
- [x] Fix Clippy warnings
- [x] Add README documentation
- [x] Search mode for tasks
- [x] Theme customization (Dayroll, Nord, Gruvbox, Tokyo Night)

### Planned
- [ ] Tags and categories
- [ ] Recurring tasks
- [ ] Statistics and analytics
- [ ] Export/Import functionality
- [ ] Sync with external services

### Future Considerations
- [ ] Keyboard macros
- [ ] Task templates
- [ ] Command palette
- [ ] Plugin system
- [ ] Calendar integrations (low priority): Google Calendar API + iCloud CalDAV (phased: import -> export -> two-way sync)
- [ ] Dedicated overdue-rollover widget (evaluate layout: beside task list vs beside calendar panel)

## Building from Source

```bash
# Clone the repository
git clone https://github.com/highdeftant/dayroll.git
cd dayroll

# Build
cargo build --release

# Run
./target/release/dayroll
```

## License

MIT License
