use parking_lot::RwLock;

use crate::{
    key::DatabaseKeyIndex,
    runtime::WaitResult,
    zalsa::{MemoIngredientIndex, Zalsa},
    zalsa_local::ZalsaLocal,
    Database,
};

use super::util;

/// Tracks the keys that are currently being processed; used to coordinate between
/// worker threads.
#[derive(Default)]
pub struct SyncTable {
    syncs: RwLock<Vec<SyncState>>,
}

/// Morally equivalent to `Option<SyncState>` where:
/// ```ignore (demonstration)
/// struct SyncState {
///     id: ThreadId,
///
///     /// Set to true if any other queries are blocked,
///     /// waiting for this query to complete.
///     anyone_waiting: bool,
/// }
/// ```
///
/// But the above struct weighs 16 bytes - `ThreadId` is 8 bytes, `AtomicBool` is 1 and 7 bytes
/// for padding, while realistically, maybe std needs to care about 2^64-1 threads - we will
/// be very fine with 2^15-1, the MSB can encode `anyone_waiting`, and zero can be reserved for
/// `None`.
#[derive(Clone, Copy)]
struct SyncState(u16);

impl Default for SyncState {
    fn default() -> Self {
        SyncState(0)
    }
}

impl SyncState {
    const ANYONE_WAITING_BIT: u16 = 0b1000_0000_0000_0000;

    fn new(thread_id: ThreadId) -> Self {
        Self(thread_id.0)
    }

    fn is_none(self) -> bool {
        self.0 == 0
    }

    fn set_anyone_waiting(&mut self) {
        // NB: `Ordering::Relaxed` is sufficient here,
        // as there are no loads that are "gated" on this
        // value. Everything that is written is also protected
        // by a lock that must be acquired. The role of this
        // boolean is to decide *whether* to acquire the lock,
        // not to gate future atomic reads.
        self.0 |= Self::ANYONE_WAITING_BIT;
    }

    fn thread_id(&self) -> ThreadId {
        ThreadId(self.0 & !Self::ANYONE_WAITING_BIT)
    }

    fn anyone_waiting(&self) -> bool {
        (self.0 & Self::ANYONE_WAITING_BIT) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(u16);

impl ThreadId {
    pub(crate) const MAX: u16 = 0b0111_1111_1111_1111;

    pub(crate) fn from_usize(value: usize) -> Self {
        if value == 0 {
            panic!("ThreadId cannot be zero");
        }
        if value > Self::MAX as usize {
            panic!("You cannot have more than `{}` threads", Self::MAX);
        }
        Self(value as u16)
    }
}

impl SyncTable {
    pub(crate) fn claim<'me>(
        &'me self,
        db: &'me dyn Database,
        zalsa_local: &ZalsaLocal,
        database_key_index: DatabaseKeyIndex,
        memo_ingredient_index: MemoIngredientIndex,
    ) -> Option<ClaimGuard<'me>> {
        let mut syncs = self.syncs.write();
        let zalsa = db.zalsa();

        util::ensure_vec_len(&mut syncs, memo_ingredient_index.as_usize() + 1);

        let sync = &mut syncs[memo_ingredient_index.as_usize()];
        if sync.is_none() {
            *sync = SyncState::new(zalsa_local.thread_id());
            Some(ClaimGuard {
                database_key_index,
                memo_ingredient_index,
                zalsa,
                sync_table: self,
            })
        } else {
            sync.set_anyone_waiting();
            zalsa.block_on_or_unwind(db, zalsa_local, database_key_index, sync.thread_id(), syncs);
            None
        }
    }
}

/// Marks an active 'claim' in the synchronization map. The claim is
/// released when this value is dropped.
#[must_use]
pub(crate) struct ClaimGuard<'me> {
    database_key_index: DatabaseKeyIndex,
    memo_ingredient_index: MemoIngredientIndex,
    zalsa: &'me Zalsa,
    sync_table: &'me SyncTable,
}

impl ClaimGuard<'_> {
    fn remove_from_map_and_unblock_queries(&self, wait_result: WaitResult) {
        let mut syncs = self.sync_table.syncs.write();

        let sync = std::mem::take(&mut syncs[self.memo_ingredient_index.as_usize()]);

        // NB: `Ordering::Relaxed` is sufficient here,
        // see `store` above for explanation.
        if sync.anyone_waiting() {
            self.zalsa
                .unblock_queries_blocked_on(self.database_key_index, wait_result)
        }
    }
}

impl Drop for ClaimGuard<'_> {
    fn drop(&mut self) {
        let wait_result = if std::thread::panicking() {
            WaitResult::Panicked
        } else {
            WaitResult::Completed
        };
        self.remove_from_map_and_unblock_queries(wait_result)
    }
}

impl std::fmt::Debug for SyncTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncTable").finish()
    }
}
