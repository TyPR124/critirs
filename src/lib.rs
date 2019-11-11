pub(crate) mod common;
pub use common::EnteredCritical;
pub(crate) mod wrapper;

#[cfg(feature = "CriticalSection")]
mod crit;
#[cfg(feature = "CriticalSection")]
pub use crit::CriticalSection;
#[cfg(feature = "CriticalStatic")]
mod crit_static;
#[cfg(feature = "CriticalStatic")]
pub use crit_static::{CriticalStatic, CriticalStaticRef};