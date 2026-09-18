//! Synthetic interleavings; no game or render device required.
use sekiro_deflect_observer::lifecycle::Gate;
use std::time::Duration;
#[test]
fn observed_loss_rejects_snapshot_even_before_reader_can_publish_under_mutex() {
    let gate = Gate::default();
    let snapshot = gate.generation();
    gate.invalidate(Duration::from_millis(1010));
    assert!(!gate.accepts(snapshot));
    assert_eq!(gate.invalidated_at(), Duration::from_millis(1010));
}
#[test]
fn hide_then_show_between_frames_still_invalidates_pre_hide_snapshot() {
    let gate = Gate::default();
    let snapshot = gate.generation();
    gate.invalidate(Duration::from_millis(1001)); // F8 hides.
                                                  // Showing does not undo invalidation even if no render observed false.
    assert!(!gate.accepts(snapshot));
    assert!(gate.accepts(gate.generation()));
}
