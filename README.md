# Dayroll

A terminal-based task manager with a day-first calendar interface for organizing daily tasks.

## Overview

Dayroll helps you manage tasks by organizing them into **overdue** and **today** buckets. Tasks can be assigned to any date, making it easy to see what needs attention and plan ahead.

## Workflow

1. **Launch**: Dayroll opens on today's date, showing all overdue tasks and today's tasks
2. **Navigate**: Use arrow keys or j/k to move between tasks
3. **Add tasks**: Press `a` to add a new task
4. **Edit tasks**: Press `e` to edit a task's title, priority, or assigned date
5. **Move tasks**: Press `m` to move a task to a different date
6. **Complete tasks**: Press `Enter` or `Space` to mark tasks as done
7. **Delete tasks**: Press `d` to delete a task

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Left` / `Right` | Move between days |
| `Left` / `Right` (in month view) | Move between months |

### Task Management
| Key | Action |
|-----|--------|
| `q` | Quit |
| `t` | Jump to today |
| `a` | Add new task |
| `e` | Edit task (title, priority, date) |
| `m` | Move task to different date |
| `d` | Delete task |
| `Enter` / `Space` | Toggle task completion |

## Buckets

### Overdue Tasks
- All incomplete tasks assigned to past dates
- Displayed at the top for immediate attention
- Sorted by assigned date (oldest first)

### Today's Tasks
- All incomplete tasks assigned to today
- Displayed below overdue tasks
- Sorted by priority (high → medium → low)

### Priority System
- **High**: Displayed with `!` prefix (red)
- **Medium**: No prefix (default)
- **Low**: Displayed with `-` prefix (dim)

## Task Operations

### Adding a Task
1. Press `a` to open the add task dialog
2. Enter the task title and press Enter
3. Set priority (↑/↓ to cycle) and press Enter
4. Set assigned date (default is today)
5. Press Enter to save

### Editing a Task
1. Select a task and press `e`
2. Edit title, priority, or date fields
3. Press Enter to save changes

### Moving a Task
1. Select a task and press `m`
2. Choose a new date from the calendar modal
3. Task is reassigned to the selected date

### Deleting a Task
1. Select a task and press `d`
2. Confirm deletion

## Storage

### Location
Tasks are stored in `~/.dayroll/todos.json`

### Format
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Complete project report",
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
- `status`: "pending" or "done"
- `priority`: "low", "medium", or "high"
- `assigned_day`: Target date in ISO format (YYYY-MM-DD)
- `completed_at`: ISO timestamp when completed (null if pending)

## Roadmap

### In Progress
- [x] Fix Clippy warnings
- [x] Add README documentation
- [ ] Search/filter for tasks
- [ ] Dark mode / theme customization

### Planned
- [ ] Tags and categories
- [ ] Task notes/descriptions
- [ ] Recurring tasks
- [ ] Statistics and analytics
- [ ] Export/Import functionality
- [ ] Sync with external services

### Future Considerations
- [ ] Keyboard macros
- [ ] Task templates
- [ ] Multiple calendars
- [ ] Command palette
- [ ] Plugin system

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/dayroll.git
cd dayroll

# Build
cargo build --release

# Run
./target/release/dayroll
```

## License

MIT License
