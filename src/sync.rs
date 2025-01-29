#[cfg(all(feature = "loom", test))]
pub use loom::{sync::*, thread};
#[cfg(not(all(feature = "loom", test)))]
pub use std::{sync::*, thread};
