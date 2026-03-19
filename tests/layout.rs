use agx::ui::layout::compute_layout;
use agx::SplitDirection;
use ratatui::layout::Rect;

#[test]
fn add_pane_recalculates() {
    let area = Rect::new(0, 0, 120, 40);

    let one = compute_layout(area, 1, SplitDirection::Vertical);
    let two = compute_layout(area, 2, SplitDirection::Vertical);
    let three = compute_layout(area, 3, SplitDirection::Vertical);

    assert_eq!(one.pane_areas[0].width, 120);
    assert_eq!(two.pane_areas[0].width, 60);
    assert_eq!(two.pane_areas[1].width, 60);
    assert_eq!(three.pane_areas[0].width, 40);
    assert_eq!(three.pane_areas[1].width, 40);
    assert_eq!(three.pane_areas[2].width, 40);
}

#[test]
fn remove_pane_recalculates() {
    let area = Rect::new(0, 0, 120, 40);

    let three = compute_layout(area, 3, SplitDirection::Vertical);
    let two = compute_layout(area, 2, SplitDirection::Vertical);

    assert_eq!(three.pane_areas[0].width, 40);
    assert_eq!(two.pane_areas[0].width, 60);
    assert_eq!(two.pane_areas[1].width, 60);
}
