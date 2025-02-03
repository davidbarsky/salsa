#[cfg(feature = "loom")]
pub use loom::{cell, sync::*, thread, thread_local};

#[cfg(not(feature = "loom"))]
pub use std::{cell, sync::*, thread, thread_local};

#[cfg(not(feature = "loom"))]
pub use crossbeam::atomic::AtomicCell;

#[cfg(feature = "loom")]
pub use crossbeam::atomic::AtomicCell;

#[cfg(not(feature = "loom"))]
pub use arc_swap::ArcSwap;

#[cfg(feature = "loom")]
mod arc_swap {
    use super::{Arc, RwLock};

    /// Mock implementation of `arc_swap::ArcSwap`.
    pub struct ArcSwap<T> {
        inner: RwLock<Arc<T>>,
    }

    impl<T> ArcSwap<T> {
        pub fn new(inner: Arc<T>) -> Self {
            let inner = RwLock::new(inner);
            Self { inner }
        }

        pub fn swap(&self, new: Arc<T>) -> Arc<T> {
            match self.inner.write() {
                Ok(mut guard) => std::mem::replace(&mut *guard, new),
                _ => panic!("lock poisoned"),
            }
        }

        pub fn into_inner(self) -> Arc<T> {
            self.inner.into_inner().unwrap()
        }

        pub fn load_full(&self) -> Arc<T> {
            self.inner.read().unwrap().clone()
        }
    }
}

#[cfg(feature = "loom")]
pub use arc_swap::ArcSwap;
