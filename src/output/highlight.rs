use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::config::manager::HighlightRule;

/// Apply highlight rules to a single line of text, producing a styled `Line`.
pub fn highlight_line<'a>(
    text: &'a str,
    rules: &'a [HighlightRule],
    _search_fg: Option<Color>,
) -> Line<'a> {
    let bytes = text.as_bytes();
    let mut styles: Vec<Style> = vec![Style::default(); bytes.len()];

    // Apply highlight rules (first matching rule wins per byte).
    for rule in rules {
        let Ok(re) = regex::Regex::new(&rule.pattern) else {
            continue;
        };
        let style = build_style(rule);
        for m in re.find_iter(text) {
            for s in styles
                .iter_mut()
                .take(m.end().min(bytes.len()))
                .skip(m.start())
            {
                if *s == Style::default() {
                    *s = style;
                }
            }
        }
    }

    // Build Spans by grouping consecutive identical styles.
    let mut result = Vec::new();
    let mut group_start = 0;
    let mut current_style = styles.first().copied().unwrap_or_default();

    for (i, &style) in styles.iter().enumerate() {
        if style != current_style {
            push_span(&mut result, &bytes[group_start..i], current_style);
            group_start = i;
            current_style = style;
        }
    }
    // Flush last group.
    if bytes.len() > group_start {
        push_span(&mut result, &bytes[group_start..], current_style);
    }

    if result.is_empty() {
        result.push(Span::raw(text));
    }

    Line::from(result)
}

fn build_style(rule: &HighlightRule) -> Style {
    let mut style = Style::default();
    if let Some(fg) = rule.fg {
        style = style.fg(fg);
    }
    if let Some(bg) = rule.bg {
        style = style.bg(bg);
    }
    if rule.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if rule.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn push_span<'a>(spans: &mut Vec<Span<'a>>, bytes: &'a [u8], style: Style) {
    let Ok(content) = std::str::from_utf8(bytes) else {
        return;
    };
    if content.is_empty() {
        return;
    }
    if style == Style::default() {
        spans.push(Span::raw(content.to_string()));
    } else {
        spans.push(Span::styled(content.to_string(), style));
    }
}
