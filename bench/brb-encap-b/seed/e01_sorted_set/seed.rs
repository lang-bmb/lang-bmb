use solution::SortedSet;

fn values(s: &SortedSet) -> Vec<i64> {
    (0..s.size()).map(|i| s.at(i).unwrap()).collect()
}

#[test]
fn insert_keeps_order_and_rejects_duplicates() {
    let mut s = SortedSet::new();
    assert!(s.insert(5));
    assert!(s.insert(3));
    assert!(s.insert(9));
    assert!(!s.insert(3));
    assert_eq!(values(&s), vec![3, 5, 9]);
    assert!(s.contains(5));
    assert!(!s.contains(4));
}

#[test]
fn insert_stops_at_eight() {
    let mut s = SortedSet::new();
    for x in 0..8 {
        assert!(s.insert(10 - x));
    }
    assert!(!s.insert(100));
    assert_eq!(values(&s), vec![3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn remove_keeps_the_rest() {
    let mut s = SortedSet::new();
    for x in [4, 1, 7] {
        s.insert(x);
    }
    assert!(s.remove(4));
    assert!(!s.remove(4));
    assert_eq!(values(&s), vec![1, 7]);
    assert_eq!(s.at(2), None);
}
