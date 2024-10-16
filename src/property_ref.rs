//===========================================================
//=== PROPERTY
//===========================================================

use crate::{DeviceRef, HomieID, NodeRef, ToTopic, HOMIE_VERSION};

/// Identifies a property of a node via its NodeRef and the property id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct PropertyRef {
    /// Identifier of the node the property belongs to
    pub node: NodeRef,
    /// The property's id within the node
    pub id: HomieID,
}

impl PropertyRef {
    /// Create a new PropertyRef from a given topic_root, device id, node id, and property id
    pub fn new(topic_root: String, device_id: HomieID, node_id: HomieID, prop_id: HomieID) -> Self {
        Self {
            node: NodeRef::new(topic_root, device_id, node_id),
            id: prop_id,
        }
    }

    /// Create a new PropertyRef from an existing NodeRef and a property id
    pub fn from_node(node: NodeRef, prop_id: HomieID) -> Self {
        Self { node, id: prop_id }
    }

    /// Return a slice of the property id
    pub fn prop_id(&self) -> &HomieID {
        &self.id
    }

    /// Return a slice of the node id the property belongs to
    pub fn node_id(&self) -> &HomieID {
        &self.node.id
    }

    /// Return a slice of the device id the property belongs to
    pub fn device_id(&self) -> &HomieID {
        &self.node.device.id
    }
}

impl PartialEq<DeviceRef> for PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<DeviceRef> for &PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<&DeviceRef> for PropertyRef {
    fn eq(&self, other: &&DeviceRef) -> bool {
        &&self.node.device == other
    }
}

impl PartialEq<NodeRef> for PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        &self.node == other
    }
}

impl PartialEq<&NodeRef> for PropertyRef {
    fn eq(&self, other: &&NodeRef) -> bool {
        &&self.node == other
    }
}

impl PartialEq<NodeRef> for &PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        &self.node == other
    }
}

impl PropertyRef {
    pub fn match_with_node(&self, node: &NodeRef, prop_id: &HomieID) -> bool {
        self == node && &self.id == prop_id
    }
}

impl ToTopic for PropertyRef {
    fn to_topic(&self) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}/{}",
            self.node.device.homie_domain, self.node.device.id, self.node.id, self.id
        )
    }
}
