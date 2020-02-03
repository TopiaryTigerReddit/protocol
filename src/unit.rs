use crate::{Bottom, Channels, Protocol};
use core::{
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
};
use futures::future::{ready, Ready};

impl<C, F: ?Sized> Protocol<C, F> for () {
    type Unravel = Bottom;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Bottom;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<(), Infallible>>;

    fn unravel(self, _: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }
}

impl<T, C, F: ?Sized> Protocol<C, F> for [T; 0] {
    type Unravel = Bottom;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Bottom;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<[T; 0], Infallible>>;

    fn unravel(self, _: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok([]))
    }
}

impl<T: ?Sized, C, F: ?Sized> Protocol<C, F> for PhantomData<T> {
    type Unravel = Bottom;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Bottom;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<PhantomData<T>, Infallible>>;

    fn unravel(self, _: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(PhantomData))
    }
}

impl<C, F: ?Sized> Protocol<C, F> for PhantomPinned {
    type Unravel = Bottom;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Bottom;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<PhantomPinned, Infallible>>;

    fn unravel(self, _: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(PhantomPinned))
    }
}
