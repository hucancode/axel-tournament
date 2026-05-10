use judge::services::capacity::CapacityTracker;

#[tokio::test]
async fn test_capacity_tracker_initialization() {
    let tracker = CapacityTracker::new(5, 1000);

    assert!(tracker.can_accept_work());
    assert_eq!(tracker.claim_delay_ms(), 0);
}

#[tokio::test]
async fn test_capacity_limits() {
    let tracker = CapacityTracker::new(2, 500);

    assert!(tracker.can_accept_work());

    let _g1 = tracker.match_slot();
    assert!(tracker.can_accept_work());

    let g2 = tracker.match_slot();
    assert!(!tracker.can_accept_work());

    drop(g2);
    assert!(tracker.can_accept_work());
}

#[tokio::test]
async fn test_delay_calculation() {
    let tracker = CapacityTracker::new(10, 1000);

    assert_eq!(tracker.claim_delay_ms(), 0);

    let mut guards = Vec::new();
    for _ in 0..5 {
        guards.push(tracker.match_slot());
    }
    assert_eq!(tracker.claim_delay_ms(), 500);

    for _ in 0..5 {
        guards.push(tracker.match_slot());
    }
    assert_eq!(tracker.claim_delay_ms(), 1000);
}
