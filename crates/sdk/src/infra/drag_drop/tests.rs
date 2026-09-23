use super::*;
use gpui::{Axis, px};

fn drag() -> KeyedDrag<&'static str, u32> {
    KeyedDrag::new(EntityId::from(1), "left", vec![2, 5]).unwrap()
}

#[test]
fn capture_must_be_unique_nonempty_and_started_before_admission() {
    assert!(KeyedDrag::<_, u32>::new(EntityId::from(1), "left", vec![]).is_none());
    assert!(KeyedDrag::new(EntityId::from(1), "left", vec![1, 1]).is_none());
    let drag = drag();
    assert!(!drag.accepts(EntityId::from(1)));
    assert!(drag.cancel().is_none());
    assert_eq!(drag.start(), Some(DragDropEvent::DragStarted { source: "left", keys: vec![2, 5] }));
    assert!(drag.start().is_none());
    assert!(!drag.accepts(EntityId::from(2)));
    assert!(drag.propose(EntityId::from(2), "right", None).is_none());
    assert!(drag.propose(EntityId::from(1), "right", None).is_some());
}

#[test]
fn committed_transfer_notifies_batches_drop_and_one_end_in_order() {
    let drag = drag();
    drag.start();
    let proposal = drag.propose(EntityId::from(1), "right", Some(11)).unwrap();
    assert_eq!(proposal.keys(), &[2, 5]);
    assert_eq!(proposal.before(), Some(&11));
    assert_eq!(
        proposal.committed(true, vec![5, 2]),
        vec![
            DragDropEvent::ItemsRemoved { source: "left", target: "right", keys: vec![5, 2] },
            DragDropEvent::ItemsAdded { source: "left", target: "right", keys: vec![5, 2], before: Some(11) },
            DragDropEvent::Dropped {
                source: "left",
                target: "right",
                keys: vec![5, 2],
                before: Some(11),
                changed: true
            },
            DragDropEvent::DragEnded { source: "left", keys: vec![2, 5], reason: DragEndReason::Transferred },
        ]
    );
    assert!(!proposal.is_active());
    assert!(!drag.accepts(EntityId::from(1)));
    assert!(drag.clone().cancel().is_none());
    assert!(proposal.clone().committed(true, vec![2, 5]).is_empty());
    assert!(proposal.rejected("late rejection").is_empty());
}

#[test]
fn reorder_noop_and_rejection_have_distinct_notifications() {
    for (changed, reason) in [(true, DragEndReason::Reordered), (false, DragEndReason::Unchanged)] {
        let drag = drag();
        drag.start();
        let proposal = drag.propose(EntityId::from(1), "left", None).unwrap();
        let events = proposal.committed(changed, vec![2, 5]);
        assert_eq!(events.len(), if changed { 3 } else { 2 });
        if changed {
            assert_eq!(events[0], DragDropEvent::ItemsReordered { target: "left", keys: vec![2, 5], before: None });
        }
        assert_eq!(
            events[events.len() - 2],
            DragDropEvent::Dropped { source: "left", target: "left", keys: vec![2, 5], before: None, changed }
        );
        assert_eq!(events.last(), Some(&DragDropEvent::DragEnded { source: "left", keys: vec![2, 5], reason }));
    }
    let drag = drag();
    drag.start();
    let proposal = drag.propose(EntityId::from(1), "right", Some(99)).unwrap();
    assert_eq!(
        proposal.rejected("missing anchor"),
        vec![
            DragDropEvent::DropRejected {
                source: "left",
                target: "right",
                keys: vec![2, 5],
                before: Some(99),
                error: "missing anchor".into()
            },
            DragDropEvent::DragEnded { source: "left", keys: vec![2, 5], reason: DragEndReason::Rejected },
        ]
    );
    assert!(drag.cancel().is_none());
}

#[test]
fn cancelled_session_invalidates_every_outstanding_proposal() {
    let drag = drag();
    drag.start();
    let proposal = drag.propose(EntityId::from(1), "right", None).unwrap();
    assert_eq!(
        drag.clone().cancel(),
        Some(DragDropEvent::DragEnded { source: "left", keys: vec![2, 5], reason: DragEndReason::Cancelled })
    );
    assert!(drag.cancel().is_none());
    assert!(!proposal.is_active());
    assert!(proposal.committed(true, vec![2, 5]).is_empty());
    assert!(drag.propose(EntityId::from(1), "right", None).is_none());
}

#[test]
fn gap_geometry_covers_spacing_and_retains_marker_inside_viewport_edge() {
    for axis in [Axis::Vertical, Axis::Horizontal] {
        let zone = DropZone {
            axis,
            edge: DropEdge::Before,
            item_extent: px(36.0),
            following_gap: px(4.0),
            marker_width: px(2.0),
        };
        assert_eq!(zone.dimensions(), (px(18.0), px(0.0), px(2.0)));
        assert_eq!(DropZone { edge: DropEdge::After, ..zone }.dimensions(), (px(22.0), px(4.0), px(6.0)));
        assert_eq!(
            DropZone { edge: DropEdge::After, following_gap: px(0.0), ..zone }.dimensions(),
            (px(18.0), px(0.0), px(2.0))
        );
        assert_eq!(
            DropZone { item_extent: px(-1.0), following_gap: px(f32::NAN), marker_width: px(-2.0), ..zone }
                .dimensions(),
            (px(0.0), px(0.0), px(0.0))
        );
    }
}
