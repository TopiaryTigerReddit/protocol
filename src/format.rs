use core::{convert::Infallible, future::Future};
use core_futures_io::{AsyncRead, AsyncWrite};
use futures::future::{ready, Ready};

pub trait Format<T> {}

pub trait ItemFormat<T>: Format<T> {
    type Representation;
    type SerializeError;
    type Serialize: Future<Output = Result<Self::Representation, Self::SerializeError>>;
    type DeserializeError;
    type Deserialize: Future<Output = Result<T, Self::DeserializeError>>;

    fn serialize(&mut self, data: T) -> Self::Serialize;
    fn deserialize(&mut self, data: Self::Representation) -> Self::Deserialize;
}

pub trait ByteFormat<T>: Format<T> {
    type SerializeError;
    type Serialize: Future<Output = Result<(), Self::SerializeError>>;
    type DeserializeError;
    type Deserialize: Future<Output = Result<T, Self::DeserializeError>>;

    fn serialize<W: AsyncWrite>(&mut self, writer: &mut W, data: T) -> Self::Serialize;
    fn deserialize<R: AsyncRead>(&mut self, reader: &mut R) -> Self::Deserialize;
}

pub struct Null;

impl<T> Format<T> for Null {}

impl<T> ItemFormat<T> for Null {
    type Representation = T;
    type SerializeError = Infallible;
    type Serialize = Ready<Result<T, Infallible>>;
    type DeserializeError = Infallible;
    type Deserialize = Ready<Result<T, Infallible>>;

    fn serialize(&mut self, data: T) -> Self::Serialize {
        ready(Ok(data))
    }

    fn deserialize(&mut self, data: T) -> Self::Serialize {
        ready(Ok(data))
    }
}
