pub mod serde;
pub use self::serde::Serde;
use core::{convert::Infallible, future::Future};
use futures::{
    future::{ready, Either, Map, MapErr, Ready},
    FutureExt, TryFutureExt,
};

pub trait Format<T> {
    type Representation;
    type SerializeError;
    type Serialize: Future<Output = Result<Self::Representation, Self::SerializeError>>;
    type DeserializeError;
    type Deserialize: Future<Output = Result<T, Self::DeserializeError>>;

    fn serialize(&mut self, data: T) -> Self::Serialize;
    fn deserialize(&mut self, data: Self::Representation) -> Self::Deserialize;
}

pub struct Null;

impl<T> Format<T> for Null {
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

#[cfg(feature = "alloc")]
/// Provides an adapter from formats with byte-isomorphic representations to a byte format
///
/// *This requires the `alloc` feature, which is enabled by default*
pub mod as_bytes {
    use super::*;
    use alloc::{
        string::{FromUtf8Error, String},
        vec::Vec,
    };

    /// Adapts formats with byte-isomorphic representations to a byte format
    ///
    /// *This requires the `alloc` feature, which is enabled by default*
    pub struct AsBytes<T>(T);

    /// Defines the fallible conversion from some representation into bytes
    ///
    /// *This requires the `alloc` feature, which is enabled by default*
    pub trait TryIntoBytes {
        type Error;

        fn try_into_bytes(self) -> Result<Vec<u8>, Self::Error>;
    }

    /// *This requires the `alloc` feature, which is enabled by default*
    impl<T: AsRef<[u8]>> TryIntoBytes for T {
        type Error = Infallible;

        fn try_into_bytes(self) -> Result<Vec<u8>, Self::Error> {
            Ok(self.as_ref().into())
        }
    }

    /// Defines the fallible conversion from bytes to some representation
    ///
    /// *This requires the `alloc` feature, which is enabled by default*
    pub trait TryFromBytes: Sized {
        type Error;

        fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Error>;
    }

    /// *This requires the `alloc` feature, which is enabled by default*
    impl TryFromBytes for String {
        type Error = FromUtf8Error;

        fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Error> {
            String::from_utf8(bytes)
        }
    }

    /// Error type returned from AsBytes conversions
    ///
    /// *This requires the `alloc` feature, which is enabled by default*
    #[derive(Debug)]
    pub enum AsBytesError<T, U> {
        AsBytes(T),
        Underlying(U),
    }

    /// *This requires the `alloc` feature, which is enabled by default*
    impl<U, T: Format<U>> Format<U> for AsBytes<T>
    where
        T::Representation: TryIntoBytes + TryFromBytes,
    {
        type Representation = Vec<u8>;
        type SerializeError =
            AsBytesError<<T::Representation as TryIntoBytes>::Error, T::SerializeError>;
        type Serialize = Map<
            T::Serialize,
            fn(
                <T::Serialize as Future>::Output,
            ) -> Result<
                Vec<u8>,
                AsBytesError<<T::Representation as TryIntoBytes>::Error, T::SerializeError>,
            >,
        >;
        type DeserializeError =
            AsBytesError<<T::Representation as TryFromBytes>::Error, T::DeserializeError>;
        type Deserialize = Either<
            Ready<
                Result<
                    U,
                    AsBytesError<<T::Representation as TryFromBytes>::Error, T::DeserializeError>,
                >,
            >,
            MapErr<
                T::Deserialize,
                fn(
                    T::DeserializeError,
                )
                    -> AsBytesError<<T::Representation as TryFromBytes>::Error, T::DeserializeError>,
            >,
        >;

        fn serialize(&mut self, data: U) -> Self::Serialize {
            fn map<T: TryIntoBytes, U>(
                input: Result<T, U>,
            ) -> Result<Vec<u8>, AsBytesError<T::Error, U>> {
                let data = input.map_err(AsBytesError::Underlying)?;
                data.try_into_bytes().map_err(AsBytesError::AsBytes)
            }

            self.0
                .serialize(data)
                .map(map::<T::Representation, T::SerializeError>)
        }

        fn deserialize(&mut self, data: Vec<u8>) -> Self::Deserialize {
            fn map<T, U>(item: U) -> AsBytesError<T, U> {
                AsBytesError::Underlying(item)
            }

            match T::Representation::try_from_bytes(data) {
                Ok(data) => Either::Right(self.0.deserialize(data).map_err(
                    map::<<T::Representation as TryFromBytes>::Error, T::DeserializeError>,
                )),
                Err(e) => Either::Left(ready(Err(AsBytesError::AsBytes(e)))),
            }
        }
    }
}
