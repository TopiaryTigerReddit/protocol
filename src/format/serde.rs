use super::Format;
use crate::Bottom;
use futures::future::{ready, Ready};
use serde::{de::DeserializeOwned, Serialize};

#[macro_export]
macro_rules! Serde {
    ($item:item) => {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        $item
    };
}

pub trait Serializer {
    type SerializeError;
    type DeserializeError;
    type Representation;

    fn serialize<T: Serialize + DeserializeOwned>(
        &mut self,
        item: T,
    ) -> Result<Self::Representation, Self::SerializeError>;

    fn deserialize<T: Serialize + DeserializeOwned>(
        &mut self,
        item: Self::Representation,
    ) -> Result<T, Self::DeserializeError>;
}

pub struct Serde<T: Serializer>(T);

impl<T: Serializer, U: Serialize + DeserializeOwned> Format<U> for Serde<T> {
    type Representation = T::Representation;
    type SerializeError = T::SerializeError;
    type Serialize = Ready<Result<T::Representation, T::SerializeError>>;
    type DeserializeError = T::DeserializeError;
    type Deserialize = Ready<Result<U, T::DeserializeError>>;

    fn serialize(&mut self, item: U) -> Self::Serialize {
        ready(self.0.serialize(item))
    }

    fn deserialize(&mut self, item: T::Representation) -> Self::Deserialize {
        ready(self.0.deserialize(item))
    }
}

impl<T: Serializer> Format<Bottom> for Serde<T> {
    type Representation = T::Representation;
    type SerializeError = T::SerializeError;
    type Serialize = Ready<Result<T::Representation, T::SerializeError>>;
    type DeserializeError = T::DeserializeError;
    type Deserialize = Ready<Result<Bottom, T::DeserializeError>>;

    fn serialize(&mut self, _: Bottom) -> Self::Serialize {
        panic!("attempted to serialize bottom type")
    }

    fn deserialize(&mut self, _: T::Representation) -> Self::Deserialize {
        panic!("attempted to deserialize bottom type")
    }
}
