use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::model::identity::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: DeserializeOwned"))]
pub struct Packet<T> {
    id: Id,
    data: T,
}

impl<T> Packet<T>
where
    T: DeserializeOwned + Serialize,
{
    pub fn new(id: Id, data: T) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn data(self) -> T {
        self.data
    }
}
