pub use std::sync::Arc;

#[cfg(feature = "loom")]
pub use loom::{sync::*, thread};

#[cfg(not(feature = "loom"))]
pub use parking_lot::*;

#[cfg(not(feature = "loom"))]
pub use std::{sync::*, thread};
