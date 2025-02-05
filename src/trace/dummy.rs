macro_rules! trace_start {
    ( $dr:ident, $name:expr, $field_name:expr) => {};
    ( $dr:ident, $name:expr, $field_name:expr) => {};
    ( $dr:ident, $name:expr) => {};
    ( $dr:ident, $name:expr) => {};
}
pub(crate) use trace_start;

macro_rules! trace_stop {
    ( $dr:ident, $value:expr, $valueType:ident) => {};
    ($dr:expr, $value:expr) => {};
    ($self:expr) => {};
}
pub(crate) use trace_stop;

macro_rules! trace_abort {
    ($self:expr) => {};
}
pub(crate) use trace_abort;

macro_rules! trace_annotate {
    ($dr:ident, $name:expr) => {};
    ($dr:ident, $name:expr) => {};
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
pub(crate) use function;
