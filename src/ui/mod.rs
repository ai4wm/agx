pub mod layout;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::layout::LayoutState;
use crate::workspace::Workspace;

pub fn render(frame: &mut Frame, app: &App, layout: &LayoutState) {
    let area = frame.area();

    if app.workspaces_empty() {
        let empty = Paragraph::new("No workspaces. Press prefix then C to create one.")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(empty, area);
        render_status_bar(frame, app, layout.status_area);
        return;
    }

    if let Some(workspace) = app.current_workspace() {
        render_workspace(frame, workspace, layout);
    }

    render_status_bar(frame, app, layout.status_area);
}

fn render_workspace(frame: &mut Frame, workspace: &Workspace, layout: &LayoutState) {
    for (index, pane) in workspace.panes.iter().enumerate() {
        let is_focused = index == workspace.focused;
        let focus_color = pane.accent_color.unwrap_or(Color::Cyan);
        let border_color = if is_focused {
            focus_color
        } else {
            Color::DarkGray
        };
        let status = if pane.is_dead() {
            "dead"
        } else if pane.is_idle() {
            "idle"
        } else {
            "live"
        };
        let title = format!(" {} [{}] ", pane.label, status);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        frame.render_widget(block, layout.pane_areas[index]);
        render_vt100_screen(frame, &pane.parser, layout.pane_inners[index]);
    }
}

fn render_vt100_screen(frame: &mut Frame, parser: &vt100::Parser, area: ratatui::layout::Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let screen = parser.screen();
    let mut lines = Vec::with_capacity(area.height as usize);

    for row in 0..area.height {
        let mut spans = Vec::with_capacity(area.width as usize);

        for col in 0..area.width {
            if let Some(cell) = screen.cell(row, col) {
                let contents = cell.contents();
                let text = if contents.is_empty() {
                    " ".to_string()
                } else {
                    contents
                };

                let mut fg = vt100_color_to_ratatui(cell.fgcolor());
                let mut bg = vt100_color_to_ratatui(cell.bgcolor());

                if cell.inverse() {
                    std::mem::swap(&mut fg, &mut bg);
                }

                let mut style = Style::default().fg(fg).bg(bg);

                if cell.bold() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic() {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }

                spans.push(Span::styled(text, style));
            } else {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(index) => Color::Indexed(index),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut spans = vec![
        Span::styled(
            " agx ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if app.prefix_is_active() {
        spans.push(Span::styled(
            format!(" PREFIX:{} ", app.prefix_label()),
            Style::default().fg(Color::Black).bg(Color::Green),
        ));
        spans.push(Span::raw(" "));
    }

    if app.confirm_close_pane {
        spans.push(Span::styled(
            " Close pane? [y/n] ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    for (index, workspace) in app.workspaces.iter().enumerate() {
        let style = if index == app.current_workspace {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };

        spans.push(Span::styled(
            format!(" {}:{} ", index + 1, workspace.name),
            style,
        ));
        spans.push(Span::raw(" "));
    }

    if let Some(workspace) = app.current_workspace() {
        spans.push(Span::styled(
            format!(" panes:{} ", workspace.panes.len()),
            Style::default().fg(Color::Gray),
        ));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled(
        format!(
            " Prefix {} then Arrows/N/X/C/1-9/Q  timeout:{}s ",
            app.prefix_label(),
            app.prefix_timeout_seconds()
        ),
        Style::default().fg(Color::Gray),
    ));

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::vt100_color_to_ratatui;

    #[test]
    fn color_default() {
        assert_eq!(vt100_color_to_ratatui(vt100::Color::Default), Color::Reset);
    }

    #[test]
    fn color_indexed() {
        assert_eq!(
            vt100_color_to_ratatui(vt100::Color::Idx(196)),
            Color::Indexed(196)
        );
    }

    #[test]
    fn color_rgb() {
        assert_eq!(
            vt100_color_to_ratatui(vt100::Color::Rgb(10, 20, 30)),
            Color::Rgb(10, 20, 30)
        );
    }
}
