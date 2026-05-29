use super::*;
use std::sync::Arc;
use std::thread;

#[test]
fn new_and_load_returns_initial_value() {
    let cell = RcuCell::new(42u32);
    let snap = cell.load();
    assert_eq!(*snap, 42);
}

#[test]
fn store_replaces_value() {
    let cell = RcuCell::new(1u32);
    cell.store(2);
    let snap = cell.load();
    assert_eq!(*snap, 2);
}

#[test]
fn update_uses_old_value() {
    let cell = RcuCell::new(10u32);
    cell.update(|old| old + 5);
    assert_eq!(*cell.load(), 15);
}

#[test]
fn snapshot_outlives_store() {
    // store 後も旧スナップショットは有効。
    let cell = RcuCell::new(vec![1u32, 2, 3]);
    let old_snap = cell.load();
    cell.store(vec![10, 20, 30]);
    // old_snap はまだ旧値を指している。
    assert_eq!(*old_snap, vec![1, 2, 3]);
    let new_snap = cell.load();
    assert_eq!(*new_snap, vec![10, 20, 30]);
}

#[test]
fn concurrent_reads_and_write() {
    let cell = Arc::new(RcuCell::new(0u64));
    let n_readers = 8;
    let n_writes = 100;

    // reader スレッド群: 値が 0 以上であることを確認し続ける
    let readers: Vec<_> = (0..n_readers)
        .map(|_| {
            let c = Arc::clone(&cell);
            thread::spawn(move || {
                for _ in 0..10_000 {
                    let v = *c.load();
                    assert!(v <= n_writes as u64, "unexpected value: {}", v);
                }
            })
        })
        .collect();

    // writer: 1 ずつインクリメント
    for i in 1..=n_writes {
        cell.store(i as u64);
    }

    for r in readers {
        r.join().unwrap();
    }

    assert_eq!(*cell.load(), n_writes as u64);
}

#[test]
fn multiple_concurrent_writers() {
    // writer が直列化されることを確認（最終的に一貫した値になる）
    let cell = Arc::new(RcuCell::new(0u32));
    let writers: Vec<_> = (0..4)
        .map(|i| {
            let c = Arc::clone(&cell);
            thread::spawn(move || {
                for _ in 0..25 {
                    c.update(|old| old + 1);
                    let _ = i; // 使用済みにする
                }
            })
        })
        .collect();

    for w in writers {
        w.join().unwrap();
    }

    // 4 スレッド * 25 回 = 100 回のインクリメント
    assert_eq!(*cell.load(), 100);
}

#[test]
fn drop_with_live_snapshot() {
    // cell を drop しても snapshot は生き続ける
    let snap = {
        let cell = RcuCell::new(String::from("hello"));
        let s = cell.load();
        // cell が drop される
        s
    };
    // snap はまだ有効
    assert_eq!(*snap, "hello");
}
