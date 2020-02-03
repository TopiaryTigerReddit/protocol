#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{future::Future, ops::DerefMut};
use futures::{Sink, Stream, TryFuture};

pub mod director;
pub use director::Director;
mod flat;
pub mod format;
mod option;
mod unit;
pub use format::Format;

pub enum Bottom {}

#[derive(Debug)]
pub enum ContextError<Context, Protocol> {
    Context(Context),
    Protocol(Protocol),
}

pub trait Join<P: Protocol<Self::Target, F>, F: ?Sized>: Dispatch {
    type Error;
    type Target;
    type Output: Future<
        Output = Result<P, ContextError<Self::Error, <P::CoalesceFuture as TryFuture>::Error>>,
    >;

    fn join(&mut self, handle: Self::Handle) -> Self::Output;
}

pub trait Spawn<P: Protocol<Self::Target, F>, F: ?Sized>: Dispatch {
    type Error;
    type Target;
    type Output: Future<
        Output = Result<
            Self::Handle,
            ContextError<Self::Error, <P::UnravelFuture as TryFuture>::Error>,
        >,
    >;

    fn spawn(&mut self, item: P) -> Self::Output;
}

pub trait Pass<
    P: Protocol<<Self as Spawn<P, F>>::Target, F> + Protocol<<Self as Join<P, F>>::Target, F>,
    F: ?Sized,
>: Spawn<P, F> + Join<P, F>
{
}

impl<
        F: ?Sized,
        P: Protocol<<Self as Spawn<P, F>>::Target, F> + Protocol<<Self as Join<P, F>>::Target, F>,
        T: Spawn<P, F> + Join<P, F>,
    > Pass<P, F> for T
{
}

pub trait Channel<T, U, S: ?Sized>: Stream<Item = T> + Sink<U> + DerefMut<Target = S> {}

pub trait Dispatch {
    type Handle;
}

pub trait Channels<Unravel, Coalesce> {
    type Unravel: Channel<Coalesce, Unravel, Self>;
    type Coalesce: Channel<Unravel, Coalesce, Self>;
}

pub trait Protocol<C: ?Sized, F: ?Sized>: Sized {
    type Unravel;
    type UnravelError;
    type UnravelFuture: Future<Output = Result<(), Self::UnravelError>>;
    type Coalesce;
    type CoalesceError;
    type CoalesceFuture: Future<Output = Result<Self, Self::CoalesceError>>;

    fn unravel(self, channel: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
        F: Format<Self::Unravel> + Format<Self::Coalesce>;

    fn coalesce(channel: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
        F: Format<Self::Unravel> + Format<Self::Coalesce>;
}
