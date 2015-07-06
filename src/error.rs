use std::{self, io};
use handlebars;

// Used to wrap the error.
macro_rules! wrapped_enum {
    ($(#[$attr:meta])* pub enum $enum_name:ident, $($enum_variant_name:ident($ty:ty),)+) => (
        $(#[$attr])*
        pub enum $enum_name { $($enum_variant_name($ty)),+ }
        $(impl From<$ty> for $enum_name {
            fn from (ty: $ty) -> Self {
                $enum_name::$enum_variant_name(ty)
            }
        })+
    );
    ($(#[$attr:meta])* enum $enum_name:ident, $($enum_variant_name:ident($ty:ty),)+) => (
        $(#[$attr])*
        enum $enum_name { $($enum_variant_name($ty)),+ }
        $(impl From<$ty> for $enum_name {
            fn from (ty: $ty) -> Self {
                $enum_name::$enum_variant_name(ty)
            }
        })+
    );
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum BlueprintError {
    Config(&'static str),
    Worker(&'static str),
}

wrapped_enum!{#[derive(Debug)] pub enum Error,
    Io(io::Error),
    Template(handlebars::TemplateError),
    Blueprint(BlueprintError),
}
