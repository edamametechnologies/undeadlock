//! Conditional dispatch to instrumented or pass-through implementations of
//! `CustomRwLock` and `CustomDashMap`.

// Instrumented: debug builds always, or release when `diagnostics` is enabled.
#[cfg(any(debug_assertions, feature = "diagnostics"))]
mod debug;

#[cfg(not(any(debug_assertions, feature = "diagnostics")))]
mod release;

#[cfg(any(debug_assertions, feature = "diagnostics"))]
pub use debug::*;

#[cfg(not(any(debug_assertions, feature = "diagnostics")))]
pub use release::*;
