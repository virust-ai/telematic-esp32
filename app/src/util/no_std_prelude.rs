//! This module provides imports needed for no_std testing with serde
//! Fixes issues with macros like stringify, concat, etc. in no_std environment

#[allow(unused_imports)]
pub use core::assert_eq;
#[allow(unused_imports)]
pub use core::concat;
#[allow(unused_imports)]
pub use core::debug_assert_eq;
#[allow(unused_imports)]
pub use core::format_args;
#[allow(unused_imports)]
pub use core::marker::Sized;
#[allow(unused_imports)]
pub use core::option::Option;
#[allow(unused_imports)]
pub use core::option::Option::{None, Some};
#[allow(unused_imports)]
pub use core::panic;
#[allow(unused_imports)]
pub use core::result::Result;
#[allow(unused_imports)]
pub use core::result::Result::{Err, Ok};
#[allow(unused_imports)]
pub use core::stringify;
#[allow(unused_imports)]
pub use core::unimplemented;
#[allow(unused_imports)]
pub use core::write;

// Make sure this module is included in the crate