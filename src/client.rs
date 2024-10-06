#[derive(Clone, PartialEq, Eq)]
pub struct LastWill {
    pub topic: String,
    pub message: Vec<u8>,
    pub qos: QoS,
    pub retain: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum QoS {
    AtLeastOnce,
    AtMostOnce,
    ExactlyOnce,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Subscription {
    pub topic: String,
    pub qos: QoS,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Publish {
    pub topic: String,
    pub retain: bool,
    pub payload: Vec<u8>,
    pub qos: QoS,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Unsubscribe {
    pub topic: String,
}
