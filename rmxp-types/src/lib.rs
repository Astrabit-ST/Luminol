#![feature(min_specialization)]

mod nil_padded;
mod parameter_type;
mod rgss_structs;

pub mod rmxp_structs;

pub use nil_padded::NilPadded;
pub use parameter_type::ParameterType;
pub use rgss_structs::{Color, Table1, Table2, Table3, Tone};
pub use rmxp_structs as rpg;
