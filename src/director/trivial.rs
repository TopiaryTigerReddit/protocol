use super::{Director, DirectorError, Empty};
use crate::{Bottom, Channel, Channels, ContextError, Dispatch, Format, Join, Protocol, Spawn};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{self, Poll},
};
use futures::{
    future::MapErr, stream::IntoStream, Sink, Stream, TryFutureExt, TryStream, TryStreamExt,
};
use void::Void;

pub struct Context<T, U>(PhantomData<(T, U)>);

pub struct Unravel<T, U>(IntoStream<U>, Context<T, U>);

pub struct Coalesce<T, U>(IntoStream<T>, Context<T, U>);

impl<T: Unpin + TryStream, U: Unpin + Sink<<T as TryStream>::Ok>> Sink<T::Ok> for Unravel<T, U> {
    type Error = <U as Sink<T::Ok>>::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(ctx)
    }

    fn start_send(mut self: core::pin::Pin<&mut Self>, item: T::Ok) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(ctx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(ctx)
    }
}

impl<T: Unpin, U: Unpin + TryStream> Stream for Unravel<T, U> {
    type Item = <IntoStream<U> as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(ctx)
    }
}

impl<T: Unpin + Sink<U::Ok> + TryStream, U: Unpin + TryStream + Sink<<T as TryStream>::Ok>>
    Sink<U::Ok> for Coalesce<T, U>
{
    type Error = <T as Sink<U::Ok>>::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(ctx)
    }

    fn start_send(mut self: core::pin::Pin<&mut Self>, item: U::Ok) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(ctx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(ctx)
    }
}

impl<T: Unpin + TryStream, U: Unpin + Stream> Stream for Coalesce<T, U> {
    type Item = <IntoStream<T> as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(ctx)
    }
}

impl<T, U> Deref for Unravel<T, U> {
    type Target = Context<T, U>;

    fn deref(&self) -> &Context<T, U> {
        &self.1
    }
}

impl<T, U> DerefMut for Unravel<T, U> {
    fn deref_mut(&mut self) -> &mut Context<T, U> {
        &mut self.1
    }
}

impl<T, U> Deref for Coalesce<T, U> {
    type Target = Context<T, U>;

    fn deref(&self) -> &Context<T, U> {
        &self.1
    }
}

impl<T, U> DerefMut for Coalesce<T, U> {
    fn deref_mut(&mut self) -> &mut Context<T, U> {
        &mut self.1
    }
}

impl<T: Unpin + TryStream + Sink<U::Ok>, U: Unpin + TryStream + Sink<<T as TryStream>::Ok>>
    Channel<U::Ok, T::Ok, Context<T, U>> for Unravel<T, U>
{
}

impl<T: Unpin + TryStream + Sink<U::Ok>, U: Unpin + TryStream + Sink<<T as TryStream>::Ok>>
    Channel<T::Ok, U::Ok, Context<T, U>> for Coalesce<T, U>
{
}

impl<T: Unpin + TryStream + Sink<U::Ok>, U: Unpin + TryStream + Sink<<T as TryStream>::Ok>>
    Channels<T::Ok, U::Ok> for Context<T, U>
{
    type Unravel = Unravel<T, U>;
    type Coalesce = Coalesce<T, U>;
}

impl<T, U> Dispatch for Context<T, U> {
    type Handle = ();
}

impl<
        F: ?Sized + Format<Bottom>,
        T,
        U,
        P: Protocol<F, Context<Empty, Empty>, Unravel = Bottom, Coalesce = Bottom>,
    > Join<P, F> for Context<T, U>
{
    type Error = Void;
    type Target = Context<Empty, Empty>;
    type Output =
        MapErr<P::CoalesceFuture, fn(P::CoalesceError) -> ContextError<Void, P::CoalesceError>>;

    fn join(&mut self, _: ()) -> Self::Output {
        P::coalesce(Coalesce(Empty::new().into_stream(), Context(PhantomData)))
            .map_err(ContextError::Protocol)
    }
}

impl<
        F: ?Sized + Format<Bottom>,
        T: Unpin,
        U: Unpin,
        P: Protocol<F, Context<Empty, Empty>, Unravel = Bottom, Coalesce = Bottom>,
    > Spawn<P, F> for Context<T, U>
{
    type Error = Void;
    type Target = Context<Empty, Empty>;
    type Output =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> ContextError<Void, P::UnravelError>>;

    fn spawn(&mut self, protocol: P) -> Self::Output {
        protocol
            .unravel(Unravel(Empty::new().into_stream(), Context(PhantomData)))
            .map_err(ContextError::Protocol)
    }
}

pub struct Trivial;

impl<
        F: ?Sized + Format<P::Unravel> + Format<P::Coalesce>,
        P: Protocol<F, Context<U, T>>,
        T: Unpin + Sink<P::Unravel> + TryStream<Ok = P::Coalesce>,
        U: Unpin + TryStream<Ok = P::Unravel> + Sink<P::Coalesce>,
    > Director<P, F, U, T> for Trivial
{
    type Context = Context<U, T>;
    type UnravelError = Void;
    type Unravel =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> DirectorError<Void, P::UnravelError>>;
    type CoalesceError = Void;
    type Coalesce =
        MapErr<P::CoalesceFuture, fn(P::CoalesceError) -> DirectorError<Void, P::CoalesceError>>;

    fn unravel(self, protocol: P, transport: T) -> Self::Unravel {
        use DirectorError::Protocol;
        protocol
            .unravel(Unravel::<U, T>(
                transport.into_stream(),
                Context(PhantomData),
            ))
            .map_err(Protocol)
    }

    fn coalesce(self, transport: U) -> Self::Coalesce {
        use DirectorError::Protocol;
        P::coalesce(Coalesce::<U, T>(
            transport.into_stream(),
            Context(PhantomData),
        ))
        .map_err(Protocol)
    }
}
