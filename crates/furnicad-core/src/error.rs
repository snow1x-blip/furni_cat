//! Error and result types shared across all FurniCAD crates.

use thiserror::Error;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, CadError>;

/// Every recoverable failure of the CAD system.
#[derive(Debug, Error)]
pub enum CadError {
    #[error("entity not found: {0}")]
    EntityNotFound(u64),

    #[error("variable not defined: `{0}`")]
    UndefinedVariable(String),

    #[error("division by zero in expression")]
    DivisionByZero,

    #[error("invalid contour: {0}")]
    InvalidContour(String),

    #[error("geometry error: {0}")]
    Geometry(String),

    #[error("kernel error: {0}")]
    Kernel(String),

    #[error("boolean operation failed: {0}")]
    Boolean(String),

    #[error("fillet/chamfer failed: {0}")]
    Fillet(String),

    #[error("hardware placement invalid: {0}")]
    Hardware(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("not implemented yet: {0}")]
    Unimplemented(&'static str),

    #[error("{0}")]
    Other(String),
}

impl CadError {
    /// Convenience constructor for [`CadError::Geometry`].
    pub fn geometry(msg: impl Into<String>) -> Self {
        CadError::Geometry(msg.into())
    }

    /// Convenience constructor for [`CadError::Other`].
    pub fn other(msg: impl Into<String>) -> Self {
        CadError::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = CadError::UndefinedVariable("width".into());
        assert_eq!(e.to_string(), "variable not defined: `width`");
    }
}
