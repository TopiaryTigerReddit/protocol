use super::{Director, DirectorError};
use crate::{Bottom, Channel, Channels, ContextError, Dispatch, Format, Join, Protocol, Spawn};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{self, Poll},
};
use futures::{future::MapErr, Sink, Stream, TryFutureExt};
use void::Void;

pub struct Context;

pub struct Empty(Context);

impl Empty {
    pub fn new() -> Self {
        Empty(Context)
    }
}

impl Sink<Bottom> for Empty {
    type Error = Void;

    fn poll_ready(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: core::pin::Pin<&mut Self>, _: Bottom) -> Result<(), Self::Error> {
        panic!("received empty type `core::convert::Bottom`")
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Stream for Empty {
    type Item = Result<Bottom, Void>;

    fn poll_next(self: Pin<&mut Self>, _: &mut task::Context) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}

impl Deref for Empty {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.0
    }
}

impl DerefMut for Empty {
    fn deref_mut(&mut self) -> &mut Context {
        &mut self.0
    }
}

impl Channel<Bottom, Bottom, Context> for Empty {}

impl Channels<Bottom, Bottom> for Context {
    type Unravel = Empty;
    type Coalesce = Empty;
}

impl Dispatch for Context {
    type Handle = ();
}

impl<F: ?Sized + Format<Bottom>, P: Protocol<F, Context, Unravel = Bottom, Coalesce = Bottom>>
    Join<P, F> for Context
{
    type Error = Void;
    type Target = Context;
    type Output =
        MapErr<P::CoalesceFuture, fn(P::CoalesceError) -> ContextError<Void, P::CoalesceError>>;

    fn join(&mut self, _: ()) -> Self::Output {
        P::coalesce(Empty(Context)).map_err(ContextError::Protocol)
    }
}

impl<F: ?Sized + Format<Bottom>, P: Protocol<F, Context, Unravel = Bottom, Coalesce = Bottom>>
    Spawn<P, F> for Context
{
    type Error = Void;
    type Target = Context;
    type Output =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> ContextError<Void, P::UnravelError>>;

    fn spawn(&mut self, protocol: P) -> Self::Output {
        protocol
            .unravel(Empty(Context))
            .map_err(ContextError::Protocol)
    }
}

pub struct Null;

impl<
        F: ?Sized + Format<Bottom>,
        P: Protocol<F, Context, Unravel = Bottom, Coalesce = Bottom>,
        U,
        T,
    > Director<P, F, U, T> for Null
{
    type Context = Context;
    type UnravelError = Void;
    type Unravel =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> DirectorError<Void, P::UnravelError>>;
    type CoalesceError = Void;
    type Coalesce =
        MapErr<P::CoalesceFuture, fn(P::CoalesceError) -> DirectorError<Void, P::CoalesceError>>;

    fn unravel(self, protocol: P, _: T) -> Self::Unravel {
        use DirectorError::Protocol;
        protocol.unravel(Empty(Context)).map_err(Protocol)
    }

    fn coalesce(self, _: U) -> Self::Coalesce {
        use DirectorError::Protocol;
        P::coalesce(Empty(Context)).map_err(Protocol)
    }
}
