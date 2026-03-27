use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

use crate::ui::sidebar::SIDEBAR_WIDTH;
use crate::SplitDirection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaneLayout {
    pub outer: Rect,
    pub tabbar: Option<Rect>,
    pub content: Rect,
}

pub struct LayoutState {
    pub sidebar: Option<Rect>,
    pub pane_layouts: Vec<PaneLayout>,
    pub status_area: Rect,
}

pub fn compute_layout(
    area: Rect,
    show_sidebar: bool,
    pane_surface_counts: &[usize],
    split: SplitDirection,
) -> LayoutState {
    let (sidebar, main_area) = if show_sidebar {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
            .split(area);
        (Some(columns[0]), columns[1])
    } else {
        (None, area)
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(main_area);

    let pane_area = sections[0];
    let status_area = sections[1];

    let pane_layouts = if pane_surface_counts.is_empty() {
        Vec::new()
    } else {
        let direction = match split {
            SplitDirection::Vertical => Direction::Horizontal,
            SplitDirection::Horizontal => Direction::Vertical,
        };
        let pane_count = pane_surface_counts.len();
        let constraints = (0..pane_count)
            .map(|_| Constraint::Ratio(1, pane_count as u32))
            .collect::<Vec<_>>();

        Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(pane_area)
            .iter()
            .copied()
            .zip(pane_surface_counts.iter().copied())
            .map(|(outer, surface_count)| pane_layout(outer, surface_count))
            .collect()
    };

    LayoutState {
        sidebar,
        pane_layouts,
        status_area,
    }
}

fn pane_layout(outer: Rect, surface_count: usize) -> PaneLayout {
    let inner = Block::default().borders(Borders::ALL).inner(outer);
    let (tabbar, content) = if surface_count >= 2 && inner.height > 0 {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (Some(sections[0]), sections[1])
    } else {
        (None, inner)
    };

    PaneLayout {
        outer,
        tabbar,
        content,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::compute_layout;
    use crate::ui::sidebar::SIDEBAR_WIDTH;
    use crate::SplitDirection;

    #[test]
    fn split_two_vertical() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 60);
        assert_eq!(layout.pane_layouts[1].outer.width, 60);
        assert_eq!(layout.pane_layouts[0].outer.height, 39);
    }

    #[test]
    fn split_single_fullscreen() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 120);
        assert_eq!(layout.pane_layouts[0].outer.height, 39);
    }

    #[test]
    fn layout_zero_panes() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[],
            SplitDirection::Vertical,
        );
        assert!(layout.pane_layouts.is_empty());
        assert_eq!(layout.status_area.height, 1);
        assert_eq!(layout.status_area.width, 120);
    }

    #[test]
    fn layout_two_panes_horizontal_split() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1],
            SplitDirection::Horizontal,
        );

        assert_eq!(layout.pane_layouts.len(), 2);
        assert_eq!(layout.pane_layouts[0].outer.width, 120);
        assert_eq!(layout.pane_layouts[1].outer.width, 120);
    }

    #[test]
    fn layout_three_panes_even_split() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 40);
        assert_eq!(layout.pane_layouts[1].outer.width, 40);
        assert_eq!(layout.pane_layouts[2].outer.width, 40);
    }

    #[test]
    fn layout_content_is_inner_without_tabbar() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert_eq!(
            layout.pane_layouts[0].content.width,
            layout.pane_layouts[0].outer.width.saturating_sub(2)
        );
        assert_eq!(
            layout.pane_layouts[0].content.height,
            layout.pane_layouts[0].outer.height.saturating_sub(2)
        );
        assert!(layout.pane_layouts[0].tabbar.is_none());
    }

    #[test]
    fn layout_status_bar_always_one_row() {
        for pane_count in 0..5 {
            let counts = vec![1; pane_count];
            let layout = compute_layout(
                Rect::new(0, 0, 80, 24),
                false,
                &counts,
                SplitDirection::Vertical,
            );
            assert_eq!(layout.status_area.height, 1);
        }
    }

    #[test]
    fn layout_with_sidebar_reserves_fixed_width() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            true,
            &[1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.sidebar.unwrap().width, SIDEBAR_WIDTH);
        assert_eq!(layout.status_area.width, 120 - SIDEBAR_WIDTH);
    }

    #[test]
    fn layout_without_sidebar_uses_full_width() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert!(layout.sidebar.is_none());
        assert_eq!(layout.status_area.width, 120);
    }

    #[test]
    fn pane_tabbar_appears_when_multiple_surfaces_exist() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[2],
            SplitDirection::Vertical,
        );

        let pane = layout.pane_layouts[0];
        assert_eq!(pane.tabbar.unwrap().height, 1);
        assert_eq!(pane.content.height, pane.outer.height.saturating_sub(3));
    }
}
