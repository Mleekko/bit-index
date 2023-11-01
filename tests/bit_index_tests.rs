use std::time::Instant;

use bit_index::BitIndex;

#[test]
fn test_bits() {
    assert_eq!(1, (0b10000000 >> 7) & 1);
    assert_eq!(0, (0b10000000 >> 1) & 1);
    assert_eq!(1, (0b00000001 >> 0) & 1);
    assert_eq!(0, (0b00000001 >> 1) & 1);
}

#[test]
fn add_and_size() {
    let mut index = BitIndex::new();

    assert_eq!(0, index.size());

    assert_eq!(0, index.reserve_slot());
    assert_eq!(1, index.size());
    assert_eq!(1, index.reserve_slot());
    assert_eq!(2, index.reserve_slot());
    assert_eq!(3, index.size());
}

#[test]
fn add_and_remove() {
    let mut index = BitIndex::new();

    for i in 0..40usize {
        index.reserve_slot();
    }

    assert_eq!(40, index.size());

    index.remove(25);

    assert_eq!(39, index.size());
    assert_eq!(26, index.find_idx_by_ordinal(25));

    assert_eq!(25, index.reserve_slot());
}

#[test]
fn usage_scenario() {
    let mut index = BitIndex::new();

    let key = index.reserve_slot();
    // KVS.insert(key, your value);
    assert_eq!(1, index.size());
    assert_eq!(0, index.find_idx_by_ordinal(0));

    // ....

    // later on:
    // KVS.remove(key)
    index.remove(key);
    assert_eq!(0, index.size());
}

#[test]
fn performance_find_remove_add() {
    let mut index = BitIndex::new();
    let size = 8192usize;
    let iterations = 8192usize;

    let start = Instant::now();

    for i in 0..size {
        index.reserve_slot();
    }

    let time1 = start.elapsed().as_millis();
    println!("Reservation Time: {} ms", time1);

    for i in 0..iterations {
        // move backwards, so test the worst pass in our algo.
        let ordinal = (size * 1000 - 7 * i) % size;

        let idx = index.find_idx_by_ordinal(ordinal);
        index.remove(idx);

        let slot = index.reserve_slot();
        assert_eq!(idx, slot);
    }

    println!("Iterations Time: {} ms", (start.elapsed().as_millis() - time1));
}