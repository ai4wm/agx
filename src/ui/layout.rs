use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

use crate::SplitDirection;

pub struct LayoutState {
    pub pane_areas: Vec<Rect>,
    pub pane_inners: Vec<Rect>,
    pub status_area: Rect,
}

pub fn compute_layout(area: Rect, pane_count: usize, split: SplitDirection) -> LayoutState {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let pane_area = sections[0];
    let status_area = sections[1];

    let pane_areas = if pane_count == 0 {
        Vec::new()
    } else {
        let direction = match split {
            SplitDirection::Vertical => Direction::Horizontal,
            SplitDirection::Horizontal => Direction::Vertical,
        };

        let constraints = (0..pane_count)
            .map(|_| Constraint::Ratio(1, pane_count as u32))
            .collect::<Vec<_>>();
        let chunks = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(pane_area);

        chunks.iter().copied().collect::<Vec<_>>()
    };

    let pane_inners = pane_areas
        .iter()
        .map(|area| Block::default().borders(Borders::ALL).inner(*area))
        .collect();

    LayoutState {
        pane_areas,
        pane_inners,
        status_area,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::compute_layout;
    use crate::SplitDirection;

    #[test]
    fn split_two_vertical() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 2, SplitDirection::Vertical);

        assert_eq!(layout.pane_areas[0].width, 60);
        assert_eq!(layout.pane_areas[1].width, 60);
        assert_eq!(layout.pane_areas[0].height, 39);
    }

    #[test]
    fn split_single_fullscreen() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 1, SplitDirection::Vertical);

        assert_eq!(layout.pane_areas[0].width, 120);
        assert_eq!(layout.pane_areas[0].height, 39);
    }
}
