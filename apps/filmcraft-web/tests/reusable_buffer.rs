#[path = "../src/reusable_buffer.rs"]
mod buffers;
use std::cell::{Cell, RefCell};

#[test]
fn overlapping_async_copies_cannot_overwrite_each_other_and_released_storage_is_reused() {
    let pool = buffers::Pool::new(16);
    let allocations = Cell::new(0);
    let make = |n| {
        allocations.set(allocations.get() + 1);
        Ok::<_, ()>(RefCell::new(vec![0u8; n as usize]))
    };
    let first = pool.acquire(4, make).unwrap();
    first.value().unwrap().borrow_mut().fill(17);
    // The first operation is suspended at an await, with its JS destination live.
    let second = pool.acquire(4, make).unwrap();
    second.value().unwrap().borrow_mut().fill(91);
    assert_eq!(*first.value().unwrap().borrow(), vec![17; 4]);
    assert_eq!(*second.value().unwrap().borrow(), vec![91; 4]);
    assert_eq!(allocations.get(), 2);
    drop(first);
    let third = pool.acquire(4, make).unwrap();
    assert_eq!(allocations.get(), 2, "reuse only after the first copy consumed its pixels");
    third.value().unwrap().borrow_mut().fill(33);
    assert_eq!(*second.value().unwrap().borrow(), vec![91; 4]);
}

#[test]
fn failed_and_oversized_operations_do_not_poison_or_expand_the_idle_cache() {
    let pool = buffers::Pool::new(16);
    assert!(pool.acquire(4, |_| Err::<Vec<u8>, _>("allocation")).is_err());
    let allocations = Cell::new(0);
    let make = |n| {
        allocations.set(allocations.get() + 1);
        Ok::<_, ()>(vec![0u8; n as usize])
    };
    drop(pool.acquire(32, make).unwrap());
    drop(pool.acquire(32, make).unwrap());
    assert_eq!(allocations.get(), 2, "oversized buffers must not be kept idle");
    drop(pool.acquire(8, make).unwrap());
    drop(pool.acquire(4, make).unwrap());
    assert_eq!(allocations.get(), 3, "a returned in-budget buffer remains reusable");
}
