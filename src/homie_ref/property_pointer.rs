use crate::HomieID;

use super::AsPropPointer;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyPointer {
    pub(crate) node_id: HomieID,
    pub(crate) prop_id: HomieID,
}

impl PropertyPointer {
    pub fn new(node_id: HomieID, prop_id: HomieID) -> Self {
        Self { node_id, prop_id }
    }
    pub fn node_id(&self) -> &HomieID {
        &self.node_id
    }
    pub fn prop_id(&self) -> &HomieID {
        &self.prop_id
    }
}

impl AsPropPointer for PropertyPointer {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self
    }
}
impl AsPropPointer for &PropertyPointer {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self
    }
}
