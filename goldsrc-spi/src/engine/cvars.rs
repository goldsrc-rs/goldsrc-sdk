//! Engine console variables (cvar) operations.

pub use crate::cvar::{CvarFlags, CvarPort};

/// Console variable operations extending the base [`CvarPort`] capability.
pub trait EngineCvars: CvarPort + Send + Sync {}

impl<T: CvarPort + Send + Sync> EngineCvars for T {}
