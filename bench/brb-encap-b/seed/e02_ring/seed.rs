use solution::Ring;

#[test]
fn first_in_first_out() {
    let mut r = Ring::new();
    assert!(r.push(1));
    assert!(r.push(2));
    assert_eq!(r.pop(), Some(1));
    assert!(r.push(3));
    assert_eq!(r.get(0), Some(2));
    assert_eq!(r.get(1), Some(3));
    assert_eq!(r.get(2), None);
    assert_eq!(r.size(), 2);
}

#[test]
fn push_stops_at_eight_and_wraps() {
    let mut r = Ring::new();
    for x in 0..8 {
        assert!(r.push(x));
    }
    assert!(!r.push(8));
    for x in 0..5 {
        assert_eq!(r.pop(), Some(x));
    }
    for x in 8..13 {
        assert!(r.push(x));
    }
    let all: Vec<i64> = (0..r.size()).map(|i| r.get(i).unwrap()).collect();
    assert_eq!(all, vec![5, 6, 7, 8, 9, 10, 11, 12]);
    assert_eq!(r.pop(), Some(5));
}

#[test]
fn pop_on_empty() {
    let mut r = Ring::new();
    assert_eq!(r.pop(), None);
}
