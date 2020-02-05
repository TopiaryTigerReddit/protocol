use core_futures_io::{AsyncRead, AsyncWrite};
use futures::{Sink, TryStream};

pub trait Format<T> {}

pub trait ItemFormat<T, S: Sink<Self::Representation> + TryStream<Ok = Self::Representation>>:
    Format<T>
{
    type Representation;
    type Output: TryStream<Ok = T> + Sink<T>;

    fn wire(self, transport: S) -> Self::Output;
}

pub trait ByteFormat<T, S: AsyncRead + AsyncWrite>: Format<T> {
    type Output: TryStream<Ok = T> + Sink<T>;

    fn wire(self, transport: S) -> Self::Output;
}

pub struct Null;

impl<T> Format<T> for Null {}

impl<T, S: Sink<T> + TryStream<Ok = T>> ItemFormat<T, S> for Null {
    type Representation = T;
    type Output = S;

    fn wire(self, transport: S) -> S {
        transport
    }
}
