use judge::services::capacity::CapacityTracker;

#[tokio::test]
async fn test_capacity_tracker_initialization() {
    let tracker = CapacityTracker::new(5, 1000);
    
    assert!(tracker.can_accept_work().await);
    assert_eq!(tracker.calculate_claim_delay().await, 0);
}

#[tokio::test]
async fn test_capacity_limits() {
    let tracker = CapacityTracker::new(2, 500);
    
    assert!(tracker.can_accept_work().await);
    
    tracker.increment_matches().await;
    assert!(tracker.can_accept_work().await);
    
    tracker.increment_matches().await;
    assert!(!tracker.can_accept_work().await);
    
    tracker.decrement_matches().await;
    assert!(tracker.can_accept_work().await);
}

#[tokio::test]
async fn test_delay_calculation() {
    let tracker = CapacityTracker::new(10, 1000);
    
    assert_eq!(tracker.calculate_claim_delay().await, 0);
    
    for _ in 0..5 {
        tracker.increment_matches().await;
    }
    let delay = tracker.calculate_claim_delay().await;
    assert_eq!(delay, 500);
    
    for _ in 0..5 {
        tracker.increment_matches().await;
    }
    let delay = tracker.calculate_claim_delay().await;
    assert_eq!(delay, 1000);
}
