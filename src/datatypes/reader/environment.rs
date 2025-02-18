use paste::paste;
use serde::Serialize;

#[derive(Serialize, Clone, Debug, Default)]
pub enum Environment {
    #[default]
    None,
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
}

impl From<Environment> for usize {
    fn from(value: Environment) -> usize {
        match value {
            Environment::None => panic!("hits cant be happening"),
            Environment::Int(i) => i as usize,
            Environment::String(_) => panic!("hits cant be happening"),
            Environment::UInt(u) => u as usize,
            Environment::Float(_) => panic!("we need to handle this better"),
        }
    }
}

impl From<Environment> for u64 {
    fn from(value: Environment) -> u64 {
        match value {
            Environment::Int(_)
            | Environment::Float(_)
            | Environment::String(_)
            | Environment::None => panic!("hits cant be happening"),
            Environment::UInt(u) => u,
        }
    }
}

pub trait IntoEnvironment {
    fn into(self) -> Environment;
}

impl IntoEnvironment for String {
    fn into(self) -> Environment {
        Environment::String(self.clone())
    }
}

macro_rules! IntoEnvironmentGenerate {
    ($first:expr, $second:expr, $($rest:expr),*) => {
    $(
    paste!{
        impl IntoEnvironment for $rest {
            fn into(self) -> Environment {
                Environment::$second(self as $first)
            }
        }
    })*
    }
}

IntoEnvironmentGenerate!(u64, UInt, u8, u16, u32, u64);
IntoEnvironmentGenerate!(i64, Int, i8, i16, i32, i64);
IntoEnvironmentGenerate!(f64, Float, f32, f64);
