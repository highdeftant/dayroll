use dayroll::app::Overlay;
use dayroll::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::markdown::render_markdown;
use crate::ui_state::{ModalState, TaskFormField};

use super::{border_style, chip_style};

pub(super) fn draw_overlay(frame: &mut ratatui::Frame<'_>, overlay: Overlay, theme: &Theme) {
    match overlay {
        Overlay::None => {}
        Overlay::Help => {
            render_scrim(frame, theme);
            let area = centered_rect(68, 60, 68, 14, frame.area());
            frame.render_widget(Clear, area);
            let text = vec![
                help_header_line(theme),
                Line::from(""),
                help_line("j/k or arrows", "move selection", theme),
                help_line("[/] or arrows", "previous / next day", theme),
                help_line("{/} or H/L", "previous / next month", theme),
                help_line("a e m d", "add, edit, move, delete", theme),
                help_line("Enter / Space", "toggle done", theme),
                help_line("/", "search mode", theme),
                help_line("u", "undo last action", theme),
                help_line("T", "next theme", theme),
                help_line("Y", "previous theme", theme),
                help_line("q", "quit confirm", theme),
                help_line("Esc", "close overlay", theme),
            ];
            let widget = Paragraph::new(text)
                .style(Style::default().fg(theme.text).bg(theme.panel))
                .block(
                    Block::default()
                        .title(" HELP ")
                        .borders(Borders::ALL)
                        .border_style(border_style(theme)),
                );
            frame.render_widget(widget, area);
        }
        Overlay::QuitConfirm => {
            render_scrim(frame, theme);
            let area = centered_rect(42, 20, 34, 9, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Quit Dayroll?",
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                help_line("y", "confirm", theme),
                help_line("n / Esc", "cancel", theme),
            ])
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .block(
                Block::default()
                    .title(Span::styled(
                        " QUIT ",
                        Style::default()
                            .fg(theme.danger)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(border_style(theme)),
            );
            frame.render_widget(widget, area);
        }
    }
}

pub(super) fn draw_modal(frame: &mut ratatui::Frame<'_>, modal: &ModalState, theme: &Theme) {
    match modal {
        ModalState::None => {}
        ModalState::MoveDate(state) => {
            render_scrim(frame, theme);
            let area = centered_rect(58, 30, 48, 11, frame.area());
            frame.render_widget(Clear, area);
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Move task date",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" target ", chip_style(theme.text, theme.info)),
                    Span::raw(" "),
                    Span::styled(state.date.to_string(), Style::default().fg(theme.text)),
                ]),
                Line::from(""),
                help_line("←/→", "day", theme),
                help_line("↑/↓", "week", theme),
                help_line("{/}", "month", theme),
                help_line("Enter", "apply", theme),
                help_line("Esc", "cancel", theme),
            ])
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .block(
                Block::default()
                    .title(" DATE PICKER ")
                    .borders(Borders::ALL)
                    .border_style(border_style(theme)),
            );
            frame.render_widget(widget, area);
        }
        ModalState::DescriptionEditor(state) => {
            render_scrim(frame, theme);
            let area = centered_rect(88, 78, 90, 22, frame.area());
            frame.render_widget(Clear, area);

            let outer = Block::default()
                .title(" NOTES ")
                .borders(Borders::ALL)
                .border_style(border_style(theme))
                .style(Style::default().bg(theme.panel));
            let inner = outer.inner(area);
            frame.render_widget(outer, area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Min(8),
                    Constraint::Length(4),
                ])
                .split(inner);
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rows[1]);

            let header = Paragraph::new(vec![Line::from(vec![
                Span::styled(" editor ", chip_style(theme.bg, theme.accent)),
                Span::raw(" raw markdown left, rendered preview right"),
            ])])
            .style(Style::default().fg(theme.text).bg(theme.panel));
            frame.render_widget(header, rows[0]);

            let editor = Paragraph::new(if state.draft.is_empty() {
                Text::from(vec![Line::from(Span::styled(
                    "<empty note>",
                    Style::default().fg(theme.muted),
                ))])
            } else {
                Text::from(state.draft.clone())
            })
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(" RAW ")
                    .borders(Borders::ALL)
                    .border_style(border_style(theme)),
            );
            frame.render_widget(editor, panes[0]);

            let preview = Paragraph::new(rendered_description_text(&state.draft, theme))
                .style(Style::default().fg(theme.text).bg(theme.panel))
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title(" PREVIEW ")
                        .borders(Borders::ALL)
                        .border_style(border_style(theme)),
                );
            frame.render_widget(preview, panes[1]);

            let footer = Paragraph::new(vec![
                help_line("Enter", "newline", theme),
                help_line("Tab", "insert spaces", theme),
                help_line("F2", "apply to task form", theme),
                help_line("Esc", "cancel", theme),
            ])
            .style(Style::default().fg(theme.text).bg(theme.panel));
            frame.render_widget(
                footer,
                rows[2].inner(Margin {
                    vertical: 0,
                    horizontal: 0,
                }),
            );
        }
        ModalState::TaskForm(form) => {
            render_scrim(frame, theme);
            let area = centered_rect(72, 46, 64, 15, frame.area());
            frame.render_widget(Clear, area);

            let title_label = field_label_style(form.field == TaskFormField::Title, theme);
            let prio_label = field_label_style(form.field == TaskFormField::Priority, theme);
            let date_label = field_label_style(form.field == TaskFormField::Date, theme);
            let desc_label = field_label_style(form.field == TaskFormField::Description, theme);

            let mut lines = vec![
                Line::from(Span::styled(
                    if form.todo_id.is_some() {
                        "Edit task"
                    } else {
                        "Add task"
                    },
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" title ", title_label),
                    Span::raw(format!(" {}", form.title)),
                ]),
                Line::from(vec![
                    Span::styled(" priority ", prio_label),
                    Span::raw(format!(" {:?}  (←/→)", form.priority)),
                ]),
                Line::from(vec![
                    Span::styled(" date ", date_label),
                    Span::raw(format!(" {}  (←/→ day, ↑/↓ week, {{/}} month)", form.date)),
                ]),
                Line::from(vec![
                    Span::styled(" description ", desc_label),
                    Span::raw(format!(" {}", description_preview(&form.description))),
                ]),
                Line::from(""),
                help_line("Tab / Shift+Tab", "cycle field", theme),
                help_line("Enter", "save task / open notes", theme),
                help_line("F2 in notes modal", "apply markdown draft", theme),
                help_line("Esc", "cancel", theme),
            ];

            if let Some(error) = &form.error {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("error: {error}"),
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            let widget = Paragraph::new(lines)
                .style(Style::default().fg(theme.text).bg(theme.panel))
                .block(
                    Block::default()
                        .title(" TASK FORM ")
                        .borders(Borders::ALL)
                        .border_style(border_style(theme)),
                );
            frame.render_widget(widget, area);
        }
    }
}

fn render_scrim(frame: &mut ratatui::Frame<'_>, theme: &Theme) {
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        frame.area(),
    );
}

fn help_line(key: &str, label: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", key),
            chip_style(theme.text, Color::Rgb(61, 73, 88)),
        ),
        Span::raw("  "),
        Span::styled(label.to_string(), Style::default().fg(theme.text)),
    ])
}

fn help_header_line(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {:<18} ", "Keyboard"),
            chip_style(theme.text, Color::Rgb(80, 94, 112)),
        ),
        Span::raw("  "),
        Span::styled(
            "Info",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn rendered_description_text(description: &str, theme: &Theme) -> Text<'static> {
    if description.trim().is_empty() {
        return Text::from(vec![Line::from(Span::styled(
            "markdown preview will appear here",
            Style::default().fg(theme.muted),
        ))]);
    }

    let mut rendered = render_markdown(description);
    for line in &mut rendered.lines {
        for span in &mut line.spans {
            span.style = Style::default().fg(theme.text).patch(span.style);
        }
    }
    rendered
}

fn description_preview(description: &str) -> String {
    if description.is_empty() {
        return "<empty>".to_string();
    }
    let compact = description.lines().collect::<Vec<_>>().join(" ⏎ ");
    let max_len = 64;
    if compact.chars().count() > max_len {
        format!("{}...", compact.chars().take(max_len).collect::<String>())
    } else {
        compact
    }
}

fn field_label_style(active: bool, theme: &Theme) -> Style {
    if active {
        chip_style(theme.bg, theme.accent)
    } else {
        chip_style(theme.text, theme.bar)
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    min_width: u16,
    min_height: u16,
    area: Rect,
) -> Rect {
    let desired_w = area.width.saturating_mul(percent_x).saturating_div(100);
    let desired_h = area.height.saturating_mul(percent_y).saturating_div(100);

    let width = desired_w.max(min_width).min(area.width.saturating_sub(2));
    let height = desired_h.max(min_height).min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.max(1),
        height: height.max(1),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use dayroll::model::Priority;
    use dayroll::theme::{ThemeName, theme_by_name};
    use ratatui::{Terminal, backend::TestBackend};

    use crate::ui_state::{DescriptionEditorState, ModalState, TaskFormField, TaskFormState};

    use super::{draw_modal, rendered_description_text};

    fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, String> {
        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("invalid date: {year:04}-{month:02}-{day:02}"))
    }

    fn task_form(day: NaiveDate) -> TaskFormState {
        TaskFormState {
            todo_id: None,
            title: "draft title".to_string(),
            priority: Priority::Medium,
            date: day,
            description: String::new(),
            field: TaskFormField::Title,
            error: None,
        }
    }

    fn render_modal_text(modal: &ModalState) -> Result<String, String> {
        let backend = TestBackend::new(120, 36);
        let mut terminal =
            Terminal::new(backend).map_err(|error| format!("terminal init failed: {error}"))?;
        let theme = theme_by_name(ThemeName::Dayroll);
        terminal
            .draw(|frame| draw_modal(frame, modal, &theme))
            .map_err(|error| format!("render failed: {error}"))?;

        let buffer = terminal.backend().buffer();
        let width = usize::from(buffer.area.width);
        let mut lines = Vec::new();
        for row in buffer.content().chunks(width) {
            let line = row
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim_end()
                .to_string();
            lines.push(line);
        }
        Ok(lines.join("\n"))
    }

    #[test]
    fn description_editor_modal_renders_titles_and_footer_help() -> Result<(), String> {
        let modal = ModalState::DescriptionEditor(DescriptionEditorState {
            parent: task_form(date(2026, 4, 18)?),
            draft: "# Heading\n- item".to_string(),
        });

        let rendered = render_modal_text(&modal)?;

        for needle in [
            "NOTES",
            "RAW",
            "PREVIEW",
            "Enter",
            "newline",
            "Tab",
            "insert spaces",
            "F2",
            "apply to task form",
            "Esc",
            "cancel",
        ] {
            assert!(
                rendered.contains(needle),
                "missing {needle:?} in rendered modal:\n{rendered}"
            );
        }

        Ok(())
    }

    #[test]
    fn description_editor_empty_state_renders_placeholders() -> Result<(), String> {
        let modal = ModalState::DescriptionEditor(DescriptionEditorState {
            parent: task_form(date(2026, 4, 18)?),
            draft: String::new(),
        });
        let theme = theme_by_name(ThemeName::Dayroll);
        let preview = rendered_description_text("", &theme);
        let preview_text = preview
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();
        let rendered = render_modal_text(&modal)?;

        assert!(rendered.contains("<empty note>"), "{rendered}");
        assert!(preview_text.contains("markdown preview will appear here"));
        assert!(
            rendered.contains("markdown preview will appear here"),
            "{rendered}"
        );
        Ok(())
    }

    #[test]
    fn description_editor_preview_renders_markdown_content() {
        let theme = theme_by_name(ThemeName::Dayroll);
        let rendered = rendered_description_text("# Heading\n- item\n**bold**", &theme);
        let flattened = rendered
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(flattened.contains("Heading"), "{flattened}");
        assert!(
            flattened.contains("• item") || flattened.contains("- item"),
            "{flattened}"
        );
        assert!(flattened.contains("bold"), "{flattened}");
    }

    #[test]
    fn task_form_modal_renders_updated_notes_help_text() -> Result<(), String> {
        let mut form = task_form(date(2026, 4, 18)?);
        form.field = TaskFormField::Description;
        let modal = ModalState::TaskForm(form);

        let rendered = render_modal_text(&modal)?;

        assert!(rendered.contains("save task / open notes"), "{rendered}");
        assert!(rendered.contains("apply markdown draft"), "{rendered}");
        Ok(())
    }
}
