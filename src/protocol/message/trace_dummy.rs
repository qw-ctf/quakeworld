macro_rules! trace_start {
    ($self:expr, $readahead:ident) => {};
    ($self:ident, $readahead:ident) => {};
}
pub(crate) use trace_start;

macro_rules! trace_stop {
    ($self:expr, $value:expr, $valueType:ident) => {};
    ($self:expr, $value:expr) => {};
    ($self:expr) => {};
}
pub(crate) use trace_stop;

macro_rules! trace_abort {
    ($self:expr) => {};
}
pub(crate) use trace_abort;

macro_rules! trace_annotate {
    ($self:expr, $value:expr) => {};
}
pub(crate) use trace_annotate;

macro_rules! trace_info {
    ($self:expr, $name:expr, $value:expr) => {};
}
pub(crate) use trace_info;

macro_rules! trace_lock {
    ($self:expr) => {};
}
pub(crate) use trace_lock;

macro_rules! trace_unlock {
    ($self:expr) => {};
}
pub(crate) use trace_unlock;

macro_rules! function {
    () => {{}};
}
