pub use std::sync::Arc;

#[cfg(loom)]
pub use loom::{sync::*, thread};

#[cfg(not(loom))]
pub use parking_lot::*;

#[cfg(not(loom))]
pub use std::{sync::*, thread};
