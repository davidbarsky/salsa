use std::hash::{BuildHasher, Hash};

pub(crate) type FxHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;
pub(crate) type FxIndexSet<K> = indexmap::IndexSet<K, FxHasher>;
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, FxHasher>;
pub(crate) type FxDashSet<K> = dashmap::DashSet<K, FxHasher>;
pub(crate) type FxLinkedHashSet<K> = hashlink::LinkedHashSet<K, FxHasher>;
pub(crate) type FxHashSet<K> = std::collections::HashSet<K, FxHasher>;

pub(crate) fn hash<T: Hash>(t: &T) -> u64 {
    FxHasher::default().hash_one(t)
}
