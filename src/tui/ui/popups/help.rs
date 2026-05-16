use super::*;

const HELP_POPUP_WIDTH: u16 = 70;

pub(in crate::tui::ui) fn render_help_popup(frame: &mut Frame, area: Rect, state: &DashboardState) {
    if !state.is_help_popup_open() {
        return;
    }

    let lines = help_popup_lines();
    let inner_width = HELP_POPUP_WIDTH.saturating_sub(4).max(1) as usize;
    let popup = centered_rect(area, HELP_POPUP_WIDTH, lines.len() as u16);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(help_popup_lines_styled(lines, inner_width))
            .block(panel_block("Keyboard Shortcuts", true))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn help_popup_lines() -> Vec<&'static str> {
    vec![
        // Global
        "Global:",
        "  q           Quit concord",
        "  Esc         Go back / close popup",
        "  `           Toggle debug log",
        "  ?           Show this help popup",
        "",
        // Navigation
        "Navigation:",
        "  j / ↓       Move selection down",
        "  k / ↑       Move selection up",
        "  g / Home    Jump to top",
        "  G / End     Jump to bottom",
        "  J           Scroll messages down one screen",
        "  K           Scroll messages up one screen",
        "  h / ←       Close tree node",
        "  l / →       Open tree node",
        "  Enter       Activate selected item",
        "",
        // Messages
        "Messages (with a message selected):",
        "  Space       Open message actions",
        "  y           Copy message content",
        "  r           Add reaction",
        "  R           Reply to message",
        "  d           Delete message",
        "  e           Edit message",
        "  v           View image",
        "  p           Show user profile",
        "  P           Pin message",
        "",
        // Composer
        "Composer:",
        "  i           Focus composer",
        "  Enter       Send message",
        "  Shift+Enter Insert newline",
        "  Ctrl+E      Open in external editor",
        "  Ctrl+C      Clear input",
        "  Ctrl+Backspace Remove last attachment",
        "  Esc         Close composer",
        "",
        // Leader
        "Leader (press Space):",
        "  1 / 2 / 4  Toggle Servers / Channels / Members",
        "  a           Open actions for focused target",
        "  o / Enter   Open settings",
        "  Space       Switch channels",
        "  Ctrl+C / Esc  Close leader",
        "",
        // Pane Focus
        "Pane Focus:",
        "  1           Focus Servers pane",
        "  2           Focus Channels pane",
        "  3           Focus Messages pane",
        "  4           Focus Members pane",
        "  Tab         Focus next pane",
        "  Shift+Tab   Focus previous pane",
        "  /           Focus pane filter",
        "",
        // Pane Resize
        "Pane Resize:",
        "  Alt+h / ←   Shrink focused pane",
        "  Alt+l / →   Expand focused pane",
        "",
        // Options popup
        "Settings (press o / Enter in leader):",
        "  j / k / ↑ / ↓  Navigate options",
        "  Enter / Space  Toggle selected option",
        "  q / o / Esc    Close settings",
        "",
    ]
}

fn help_popup_lines_styled(raw: Vec<&'static str>, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut lines = Vec::with_capacity(raw.len());
    for line in raw {
        if line.is_empty() {
            lines.push(Line::from(Span::raw(String::new())));
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            // Section header
            lines.push(Line::from(Span::styled(
                format!(" {line} "),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )));
            continue;
        }
        // Indented shortcut line — key in DIM, description normal
        let trimmed = line.trim_start();
        let key_end = trimmed.find([' ', '\t']).unwrap_or(trimmed.len());
        let (key, rest) = trimmed.split_at(key_end);
        let desc = rest.trim_start();
        lines.push(Line::from(vec![
            Span::styled(format!("{key:<12}"), Style::default().fg(DIM)),
            Span::raw(desc.to_owned()),
        ]));
    }
    lines
        .into_iter()
        .map(|line| truncate_line_to_display_width(line, width))
        .collect()
}
