mod device_ref;
mod node_ref;
mod property_pointer;
mod property_ref;

pub use device_ref::*;
pub use node_ref::*;
pub use property_pointer::*;
pub use property_ref::*;

use crate::HomieID;

pub trait AsNodeId {
    fn as_node_id(&self) -> &HomieID;
}
pub trait AsPropPointer {
    fn as_prop_pointer(&self) -> &PropertyPointer;
}

/// A node identifier relative to its device.
///
/// This is a semantic alias for `HomieID` that communicates intent when used as
/// a key in device-scoped collections (e.g., `HashMap<NodePointer, NodeData>`).
pub type NodePointer = HomieID;
