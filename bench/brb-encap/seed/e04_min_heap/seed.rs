use solution::Heap;

#[test]
fn peek_is_the_smallest() {
    let mut h = Heap::new();
    assert_eq!(h.peek(), None);
    for x in [5, 3, 8, 1, 9] {
        assert!(h.push(x));
    }
    assert_eq!(h.peek(), Some(1));
    assert_eq!(h.size(), 5);
}

#[test]
fn push_stops_at_eight() {
    let mut h = Heap::new();
    for x in 0..8 {
        assert!(h.push(10 - x));
    }
    assert!(!h.push(-1));
    assert_eq!(h.peek(), Some(3));
}
