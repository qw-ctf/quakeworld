#[cfg(feature = "trace")]
include!("trace_real.rs");
#[cfg(not(feature = "trace"))]
include!("trace_dummy.rs");
