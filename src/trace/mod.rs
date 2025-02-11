#[macro_use]
#[cfg(feature = "trace")]
include!("real.rs");
#[cfg(not(feature = "trace"))]
include!("dummy.rs");
