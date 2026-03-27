pub mod layout;
pub mod sidebar;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::pane::Pane;
use crate::ui::layout::LayoutState;
use crate::workspace::Workspace;

pub fn render(frame: &mut Frame, app: &App, layout: &LayoutState) {
    if let Some(sidebar_area) = layout.sidebar {
        sidebar::render(frame, app, sidebar_area);
    }

    if app.workspaces_empty() {
        let empty = Paragraph::new("No workspaces. Press Alt+C to create one.")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(empty, main_content_area(frame.area(), layout));
        render_status_bar(frame, app, layout.status_area);
        return;
    }

    if let Some(workspace) = app.current_workspace() {
        render_workspace(frame, workspace, layout);
    }

    render_status_bar(frame, app, layout.status_area);
}

fn render_workspace(frame: &mut Frame, workspace: &Workspace, layout: &LayoutState) {
    for (index, (pane, pane_layout)) in workspace
        .panes
        .iter()
        .zip(layout.pane_layouts.iter())
        .enumerate()
    {
        let is_focused = index == workspace.focused_pane;
        let current_surface = pane.current_surface();
        let focus_color = current_surface
            .and_then(|surface| surface.agent.accent_color)
            .unwrap_or(Color::Cyan);
        let border_color = if is_focused {
            focus_color
        } else {
            Color::DarkGray
        };
        let status = if current_surface.is_some_and(|surface| surface.agent.is_dead()) {
            "dead"
        } else if current_surface.is_some_and(|surface| surface.agent.is_idle()) {
            "idle"
        } else {
            "live"
        };
        let title_label = current_surface
            .map(|surface| surface.label.as_str())
            .unwrap_or("empty");
        let title = format!(" {} [{}] ", title_label, status);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        frame.render_widget(block, pane_layout.outer);

        if let Some(tabbar_area) = pane_layout.tabbar {
            render_surface_tabbar(frame, pane, tabbar_area, focus_color);
        }

        if let Some(surface) = current_surface {
            render_vt100_screen(frame, &surface.agent.parser, pane_layout.content);
        }
    }
}

fn render_surface_tabbar(
    frame: &mut Frame,
    pane: &Pane,
    area: ratatui::layout::Rect,
    accent: Color,
) {
    let mut spans = Vec::new();

    for (index, surface) in pane.surfaces.iter().enumerate() {
        let style = if index == pane.current_surface {
            Style::default()
                .fg(Color::Black)
                .bg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!("[{}] ", surface.label), style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn main_content_area(
    frame_area: ratatui::layout::Rect,
    layout: &LayoutState,
) -> ratatui::layout::Rect {
    let sidebar_width = layout.sidebar.map_or(0, |sidebar| sidebar.width);
    let main_x = frame_area.x.saturating_add(sidebar_width);
    let main_width = frame_area.width.saturating_sub(sidebar_width);
    let main_height = layout.status_area.y.saturating_sub(frame_area.y);

    ratatui::layout::Rect::new(main_x, frame_area.y, main_width, main_height)
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

    if let Some(workspace) = app.current_workspace() {
        spans.push(Span::styled(
            format!(" ws:{} ", workspace.name),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));
        spans.push(Span::raw(" "));

        spans.push(Span::styled(
            format!(" panes:{} ", workspace.panes.len()),
            Style::default().fg(Color::Gray),
        ));
        spans.push(Span::raw(" "));

        if let Some(pane) = workspace.focused_pane() {
            let current_surface = if pane.surfaces.is_empty() {
                0
            } else {
                pane.current_surface + 1
            };
            spans.push(Span::styled(
                format!(" surf:{}/{} ", current_surface, pane.surfaces.len()),
                Style::default().fg(Color::Gray),
            ));
            spans.push(Span::raw(" "));
        }
    }

    spans.push(Span::styled(
        " Alt+?:help Alt+D/S split Alt+T/W surface Alt+B sidebar Alt+Q quit ",
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
