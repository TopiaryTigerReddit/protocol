use super::{Director, DirectorError, Empty};
use crate::{Bottom, Channel, Channels, ContextError, Dispatch, Format, Join, Protocol, Spawn};
use core::{
    convert::Infallible,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{self, Poll},
};
use futures::{future::MapErr, Sink, Stream, TryFutureExt};

pub struct Context<T, U>(PhantomData<(T, U)>);

pub struct Unravel<T, U>(U, Context<T, U>);

pub struct Coalesce<T, U>(T, Context<T, U>);

impl<T: Unpin + Stream, U: Unpin + Sink<<T as Stream>::Item>> Sink<T::Item> for Unravel<T, U> {
    type Error = U::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(ctx)
    }

    fn start_send(mut self: core::pin::Pin<&mut Self>, item: T::Item) -> Result<(), Self::Error> {
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

impl<T: Unpin, U: Unpin + Stream> Stream for Unravel<T, U> {
    type Item = U::Item;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(ctx)
    }
}

impl<T: Unpin + Sink<<U as Stream>::Item> + Stream, U: Unpin + Stream + Sink<T::Item>> Sink<U::Item>
    for Coalesce<T, U>
{
    type Error = T::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(ctx)
    }

    fn start_send(mut self: core::pin::Pin<&mut Self>, item: U::Item) -> Result<(), Self::Error> {
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

impl<T: Unpin + Stream, U: Unpin + Stream> Stream for Coalesce<T, U> {
    type Item = T::Item;

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

impl<T: Unpin + Stream + Sink<U::Item>, U: Unpin + Stream + Sink<<T as Stream>::Item>>
    Channel<U::Item, T::Item, Context<T, U>> for Unravel<T, U>
{
}

impl<T: Unpin + Stream + Sink<U::Item>, U: Unpin + Stream + Sink<<T as Stream>::Item>>
    Channel<T::Item, U::Item, Context<T, U>> for Coalesce<T, U>
{
}

impl<T: Unpin + Stream + Sink<U::Item>, U: Unpin + Stream + Sink<<T as Stream>::Item>>
    Channels<T::Item, U::Item> for Context<T, U>
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
        P: Protocol<Context<Empty, Empty>, F, Unravel = Bottom, Coalesce = Bottom>,
    > Join<P, F> for Context<T, U>
{
    type Error = Infallible;
    type Target = Context<Empty, Empty>;
    type Output = MapErr<
        P::CoalesceFuture,
        fn(P::CoalesceError) -> ContextError<Infallible, P::CoalesceError>,
    >;

    fn join(&mut self, _: ()) -> Self::Output {
        P::coalesce(Coalesce(Empty::new(), Context(PhantomData))).map_err(ContextError::Protocol)
    }
}

impl<
        F: ?Sized + Format<Bottom>,
        T: Unpin,
        U: Unpin,
        P: Protocol<Context<Empty, Empty>, F, Unravel = Bottom, Coalesce = Bottom>,
    > Spawn<P, F> for Context<T, U>
{
    type Error = Infallible;
    type Target = Context<Empty, Empty>;
    type Output =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> ContextError<Infallible, P::UnravelError>>;

    fn spawn(&mut self, protocol: P) -> Self::Output {
        protocol
            .unravel(Unravel(Empty::new(), Context(PhantomData)))
            .map_err(ContextError::Protocol)
    }
}

pub struct Trivial;

impl<
        F: ?Sized + Format<P::Unravel> + Format<P::Coalesce>,
        P: Protocol<Context<U, T>, F>,
        T: Unpin + Sink<P::Unravel> + Stream<Item = P::Coalesce>,
        U: Unpin + Stream<Item = P::Unravel> + Sink<P::Coalesce>,
    > Director<P, F, U, T> for Trivial
{
    type Context = Context<U, T>;
    type UnravelError = Infallible;
    type Unravel =
        MapErr<P::UnravelFuture, fn(P::UnravelError) -> DirectorError<Infallible, P::UnravelError>>;
    type CoalesceError = Infallible;
    type Coalesce = MapErr<
        P::CoalesceFuture,
        fn(P::CoalesceError) -> DirectorError<Infallible, P::CoalesceError>,
    >;

    fn unravel(self, protocol: P, transport: T) -> Self::Unravel {
        use DirectorError::Protocol;
        protocol
            .unravel(Unravel::<U, T>(transport, Context(PhantomData)))
            .map_err(Protocol)
    }

    fn coalesce(self, transport: U) -> Self::Coalesce {
        use DirectorError::Protocol;
        P::coalesce(Coalesce::<U, T>(transport, Context(PhantomData))).map_err(Protocol)
    }
}
