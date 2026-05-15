use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};

#[derive(Debug, Clone, Copy, Default)]
struct StyleState {
    strong: bool,
    emphasis: bool,
    code: bool,
    heading: bool,
}

impl StyleState {
    fn to_style(self) -> Style {
        let mut style = Style::default();
        if self.strong || self.heading {
            style = style.add_modifier(Modifier::BOLD);
        }
        if self.emphasis {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if self.code {
            style = style.add_modifier(Modifier::REVERSED);
        }
        style
    }
}

fn push_span(lines: &mut Vec<Line<'static>>, span: Span<'static>) {
    if lines.is_empty() {
        lines.push(Line::default());
    }
    if let Some(last) = lines.last_mut() {
        last.spans.push(span);
    }
}

fn push_text(lines: &mut Vec<Line<'static>>, text: impl Into<String>, state: StyleState) {
    let text = text.into();
    if text.is_empty() {
        return;
    }
    push_span(lines, Span::styled(text, state.to_style()));
}

fn newline(lines: &mut Vec<Line<'static>>) {
    if lines.is_empty() || lines.last().is_some_and(|line| !line.spans.is_empty()) {
        lines.push(Line::default());
    }
}

pub fn render_markdown(md: &str) -> Text<'static> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(md, options);
    let mut lines = vec![Line::default()];
    let mut state = StyleState::default();

    for event in parser {
        match event {
            Event::Text(text) => push_text(&mut lines, text.to_string(), state),
            Event::Code(code) => {
                let mut code_state = state;
                code_state.code = true;
                push_text(&mut lines, code.to_string(), code_state);
            }
            Event::SoftBreak => push_text(&mut lines, " ", state),
            Event::HardBreak => newline(&mut lines),

            Event::Start(Tag::Strong) => state.strong = true,
            Event::End(TagEnd::Strong) => state.strong = false,
            Event::Start(Tag::Emphasis) => state.emphasis = true,
            Event::End(TagEnd::Emphasis) => state.emphasis = false,

            Event::Start(Tag::Heading { .. }) => {
                state.heading = true;
                newline(&mut lines);
            }
            Event::End(TagEnd::Heading(_)) => {
                state.heading = false;
                newline(&mut lines);
            }

            Event::Start(Tag::CodeBlock(_)) => {
                state.code = true;
                newline(&mut lines);
            }
            Event::End(TagEnd::CodeBlock) => {
                state.code = false;
                newline(&mut lines);
            }

            Event::Start(Tag::Paragraph) => newline(&mut lines),
            Event::End(TagEnd::Paragraph) => newline(&mut lines),

            Event::Start(Tag::Item) => push_text(&mut lines, "• ", state),
            Event::End(TagEnd::Item) => newline(&mut lines),

            Event::Rule => {
                newline(&mut lines);
                push_text(&mut lines, "---", state);
                newline(&mut lines);
            }
            _ => {}
        }
    }

    while lines.last().is_some_and(|line| line.spans.is_empty()) && lines.len() > 1 {
        lines.pop();
    }

    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::render_markdown;

    fn flatten_first_line(md: &str) -> String {
        let rendered = render_markdown(md);
        rendered
            .lines
            .first()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.to_string())
                    .collect::<String>()
            })
            .unwrap_or_default()
    }

    #[test]
    fn renders_heading_text() {
        assert_eq!(flatten_first_line("# Ship it"), "Ship it");
    }

    #[test]
    fn renders_emphasis_and_strong_content() {
        assert_eq!(flatten_first_line("**bold** _italics_"), "bold italics");
    }

    #[test]
    fn renders_inline_code_content() {
        assert_eq!(flatten_first_line("run `cargo test`"), "run cargo test");
    }
}
