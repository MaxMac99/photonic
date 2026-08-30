// derive-new 0.7 generates `Field { field: field }` constructors, which
// newer clippy flags as redundant field names. Allow at crate root until
// derive-new emits field shorthand.
#![allow(clippy::redundant_field_names)]

pub mod application;
pub mod domain;

pub use application::{QuotaConfig, QuotaManager};
pub use domain::QuotaReservation;
