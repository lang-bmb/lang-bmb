use solution::Spans;

fn spans(s: &Spans) -> Vec<(i64, i64)> {
    (0..s.count()).map(|i| (s.first(i).unwrap(), s.last(i).unwrap())).collect()
}

#[test]
fn push_appends_with_a_gap() {
    let mut s = Spans::new();
    assert!(s.push(1, 3));
    assert!(!s.push(4, 6)); // touches 1..=3
    assert!(!s.push(2, 6)); // overlaps
    assert!(s.push(5, 5));
    assert!(!s.push(9, 8)); // empty
    assert_eq!(spans(&s), vec![(1, 3), (5, 5)]);
}

#[test]
fn contains_looks_in_every_span() {
    let mut s = Spans::new();
    s.push(-5, -2);
    s.push(10, 20);
    for (x, want) in [(-6, false), (-5, true), (-2, true), (0, false), (15, true), (21, false)] {
        assert_eq!(s.contains(x), want, "{x}");
    }
}

#[test]
fn push_stops_at_eight() {
    let mut s = Spans::new();
    for i in 0..8 {
        assert!(s.push(3 * i, 3 * i + 1));
    }
    assert!(!s.push(100, 100));
    assert_eq!(s.count(), 8);
    assert_eq!(s.first(8), None);
}
