// Re-export according to alloc::prelude::v1 because it is not yet stabilized
#[allow(unused_imports)]
pub use alloc::vec;
pub use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
pub use core::prelude::v1::*;
