use crate::{Bottom, Channels, Protocol};
use core::marker::{PhantomData, PhantomPinned};
use futures::future::{ready, Ready};
use void::Void;

impl<C, F: ?Sized> Protocol<F, C> for () {
    type Unravel = Bottom;
    type UnravelError = Void;
    type UnravelFuture = Ready<Result<(), Void>>;
    type Coalesce = Bottom;
    type CoalesceError = Void;
    type CoalesceFuture = Ready<Result<(), Void>>;

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

impl<T, C, F: ?Sized> Protocol<F, C> for [T; 0] {
    type Unravel = Bottom;
    type UnravelError = Void;
    type UnravelFuture = Ready<Result<(), Void>>;
    type Coalesce = Bottom;
    type CoalesceError = Void;
    type CoalesceFuture = Ready<Result<[T; 0], Void>>;

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

impl<T: ?Sized, C, F: ?Sized> Protocol<F, C> for PhantomData<T> {
    type Unravel = Bottom;
    type UnravelError = Void;
    type UnravelFuture = Ready<Result<(), Void>>;
    type Coalesce = Bottom;
    type CoalesceError = Void;
    type CoalesceFuture = Ready<Result<PhantomData<T>, Void>>;

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

impl<C, F: ?Sized> Protocol<F, C> for PhantomPinned {
    type Unravel = Bottom;
    type UnravelError = Void;
    type UnravelFuture = Ready<Result<(), Void>>;
    type Coalesce = Bottom;
    type CoalesceError = Void;
    type CoalesceFuture = Ready<Result<PhantomPinned, Void>>;

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
