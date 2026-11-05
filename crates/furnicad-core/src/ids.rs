//! Strongly typed identifiers for every technological entity in a document.
//!
//! All ids are plain `u64` newtypes so they can be cheaply copied, hashed and
//! serialized while remaining type-safe at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU64;

/// Generates a small newtype id with `Display`, `From<u64>` and serialization.
macro_rules! define_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub u64);

        impl $name {
            /// Construct from a raw numeric value.
            #[inline]
            pub const fn from_raw(v: u64) -> Self {
                Self(v)
            }

            /// Raw numeric value behind this id.
            #[inline]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl From<u64> for $name {
            #[inline]
            fn from(v: u64) -> Self {
                Self(v)
            }
        }

        impl From<NonZeroU64> for $name {
            #[inline]
            fn from(v: NonZeroU64) -> Self {
                Self(v.get())
            }
        }
    };
}

define_id!(
    /// Identifies any entity inside [`crate::entity::EntityStore`].
    EntityId
);
define_id!(
    /// Identifies an entire document / project.
    DocumentId
);
define_id!(
    /// Identifies a panel entity (see `furnicad-panels`).
    PanelId
);
define_id!(
    /// Identifies an assembly (group of panels/hardware).
    AssemblyId
);
define_id!(
    /// Identifies a material in the material library.
    MaterialId
);
define_id!(
    /// Identifies a hardware definition in the hardware library.
    HardwareId
);
define_id!(
    /// Identifies a placed hardware instance.
    HardwareInstanceId
);
define_id!(
    /// Identifies a hole produced by drilling (присадка).
    HoleId
);
define_id!(
    /// Identifies a groove/milling feature.
    GrooveId
);
define_id!(
    /// Identifies a cutout feature.
    CutoutId
);
define_id!(
    /// Identifies an edge of a panel contour.
    EdgeId
);
define_id!(
    /// Identifies an edge band applied to an edge.
    EdgeBandId
);
define_id!(
    /// Identifies a node in the history graph.
    HistoryNodeId
);
define_id!(
    /// Identifies a sketch entity.
    SketchId
);
define_id!(
    /// Identifies an auxiliary plane.
    PlaneId
);
define_id!(
    /// Identifies an auxiliary line.
    LineId
);
define_id!(
    /// Identifies a dimension annotation.
    DimensionId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_types_and_comparable() {
        let p = PanelId(7);
        let e = EntityId(7);
        assert_eq!(p.raw(), e.raw());
        assert_eq!(p, PanelId::from_raw(7));
        assert_eq!(format!("{p}"), "PanelId(7)");
    }

    #[test]
    fn id_serde_roundtrip() {
        let id = MaterialId(42);
        let json = serde_json::to_string(&id).unwrap();
        let back: MaterialId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }
}
