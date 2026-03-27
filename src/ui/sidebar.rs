use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub const SIDEBAR_WIDTH: u16 = 18;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focused;
    let cursor = app.sidebar_cursor;

    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let mut lines = Vec::new();

    for (index, workspace) in app.workspaces.iter().enumerate() {
        let is_current = index == app.current_workspace;
        let is_cursor = focused && index == cursor;

        let (marker, name_style) = if is_cursor {
            (
                ">",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_current {
            (
                "*",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (" ", Style::default().fg(Color::White))
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{marker} {}", workspace.name),
            name_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  panes:{}", workspace.panes.len()),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no workspaces",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let title = if focused {
        " ^v:Move Enter:Select N:New D:Del "
    } else {
        " ws "
    };

    let sidebar = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(sidebar, area);
}

#[cfg(test)]
mod tests {
    use super::SIDEBAR_WIDTH;

    #[test]
    fn sidebar_width_matches_spec() {
        assert_eq!(SIDEBAR_WIDTH, 18);
    }
}
