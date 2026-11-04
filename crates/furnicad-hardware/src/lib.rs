//! `furnicad-hardware` — hardware library, mounting rules, automatic drilling
//! (присадка) and fastener validation (spec §10).

pub mod library;
pub mod patterns;
pub mod validate;

pub use library::{
    HardwareCategory, HardwareDefinition, HardwareGeometry, HardwareInstance, HardwareParameter,
    default_library,
};
pub use patterns::{HolePattern, HoleTemplate, MountingRule, PositionMode, generate_holes};
pub use validate::{Severity, ValidationResult, validate_drilling};
