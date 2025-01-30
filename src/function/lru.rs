use crate::{hash::FxLinkedHashSet, Id};

use crate::sync::Mutex;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub(super) struct Lru {
    capacity: AtomicCell<usize>,
    set: Mutex<FxLinkedHashSet<Id>>,
}

impl Lru {
    pub(super) fn record_use(&self, index: Id) -> Option<Id> {
        let capacity = self.capacity.load();

        if capacity == 0 {
            // LRU is disabled
            return None;
        }

        let mut set = self.set.lock().unwrap();
        set.insert(index);
        if set.len() > capacity {
            return set.pop_front();
        }

        None
    }

    pub(super) fn set_capacity(&self, capacity: usize) {
        self.capacity.store(capacity);

        if capacity == 0 {
            let mut set = self.set.lock().unwrap();
            *set = FxLinkedHashSet::default();
        }
    }
}
