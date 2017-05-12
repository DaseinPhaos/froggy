extern crate froggy;

use froggy::Storage;
use froggy::StreamingIterator;

#[test]
fn change_by_pointer() {
    let storage = Storage::new();
    {
        let mut s = storage.write();
        s.create(4 as i32);
    }
    let ptr = {
        let r = storage.read();
        let item = r.iter().next().unwrap();
        r.pin(&item)
    };
    assert_eq!(storage.write()[&ptr], 4);
    storage.write()[&ptr] = 350 as i32;
    assert_eq!(storage.write()[&ptr], 350);
}

#[test]
fn iterating() {
    let storage = Storage::new();
    {
        let mut s = storage.write();
        for &i in [5 as i32, 7, 4, 6, 7].iter() {
            s.create(i);
        }
    }
    assert_eq!(storage.read().iter().count(), 5);
    assert_eq!(*storage.read().iter().nth(0).unwrap(), 5);
    assert_eq!(*storage.read().iter().nth(1).unwrap(), 7);
    assert!(storage.read().iter().find(|v| **v == 4).is_some());
}

#[test]
fn iter_alive() {
    let storage = Storage::new();
    {
        let mut w = storage.write();
        for i in 0..5 {
            w.create(i * 3 as i32);
        }
    }
    assert_eq!(storage.write().iter().count(), 5);
    assert_eq!(storage.write().iter_alive().count(), 0);
    let ptr = {
        let r = storage.read();
        let item = r.iter().nth(0).unwrap();
        r.pin(&item)
    };
    assert_eq!(storage.read()[&ptr], 0);
    assert_eq!(storage.write().iter().count(), 5);
    assert_eq!(storage.write().iter_alive().count(), 1);
}

#[test]
fn pointer_iter() {
    let storage = Storage::new();
    {
        let mut w = storage.write();
        for i in 0..5 {
            w.create(i as i32);
        }
    }
    assert_eq!(storage.read().iter().count(), 5);
    let mut counter = 0;
    let mut write = storage.write();
    let mut iter = write.pointers();
    while let Some(ptr) = iter.next() {
        assert_eq!(write[ptr], counter);
        let _weak = ptr.downgrade();
        let _ptr2 = ptr.clone();
        counter += 1;
    }
    assert_eq!(counter, 5);
}

#[test]
fn weak_upgrade_downgrade() {
    let storage = Storage::new();
    let ptr = storage.write().create(1 as i32);
    let _write = storage.write();
    let weak = ptr.downgrade();
    assert_eq!(weak.upgrade().is_ok(), true);
}

#[test]
fn weak_epoch() {
    let storage = Storage::new();
    let weak = {
        let node1 = storage.write().create(1 as i32);
        assert_eq!(storage.write().iter_alive().count(), 1);
        node1.downgrade()
    };
    assert_eq!(storage.write().iter_alive().count(), 0);
    assert_eq!(weak.upgrade(), Err(froggy::UpgradeErr::DeadComponent));
    let _ptr = storage.write().create(1 as i32);
    assert_eq!(storage.write().iter_alive().count(), 1);
    assert_eq!(weak.upgrade(), Err(froggy::UpgradeErr::DeadComponent));
}
