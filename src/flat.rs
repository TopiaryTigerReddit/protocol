use crate::{
    format::{serde::Serializer, Serde},
    Bottom, Channels, Protocol,
};
use core::{
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
        AtomicU64, AtomicU8, AtomicUsize,
    },
    time::Duration,
};
use futures::{
    future::{ready, Map, Ready},
    stream::{once, Forward, Once, StreamFuture},
    FutureExt, Sink, StreamExt,
};

#[derive(Debug)]
pub struct Insufficient;

macro_rules! flat {
    ( $( $x:ty ),* ) => {
        $(
            impl<T: Serializer, C: Channels<$x, Bottom>> Protocol<C, Serde<T>> for $x
            where
                C::Unravel: Unpin,
                C::Coalesce: Unpin
            {
                type Unravel = $x;
                type UnravelError = <C::Unravel as Sink<$x>>::Error;
                type UnravelFuture =
                    Forward<Once<Ready<Result<$x, <C::Unravel as Sink<$x>>::Error>>>, C::Unravel>;
                type Coalesce = Bottom;
                type CoalesceError = Insufficient;
                type CoalesceFuture =
                    Map<StreamFuture<C::Coalesce>, fn((Option<$x>, C::Coalesce)) -> Result<$x, Insufficient>>;

                fn unravel(self, channel: C::Unravel) -> Self::UnravelFuture {
                    once(ready(Ok(self))).forward(channel)
                }

                fn coalesce(channel: C::Coalesce) -> Self::CoalesceFuture {
                    fn map<St>(next: (Option<$x>, St)) -> Result<$x, Insufficient> {
                        next.0.ok_or(Insufficient)
                    }
                    channel.into_future().map(map::<C::Coalesce>)
                }
            }
        )*
    };
    ( #[doc = $doc:literal] $( $x:ty ),* ) => {
        $(
            #[doc = $doc]
            flat!($x);
        )*
    };
}

flat! {
    bool, char, f32, f64, Duration,
    usize, u8, u16, u32, u64, u128,
    isize, i8, i16, i32, i64, i128,
    AtomicBool,
    AtomicIsize, AtomicI8, AtomicI16, AtomicI32, AtomicI64,
    AtomicUsize, AtomicU8, AtomicU16, AtomicU32, AtomicU64,
    NonZeroIsize, NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128,
    NonZeroUsize, NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128
}

#[cfg(feature = "std")]
mod std {
    use super::*;
    use ::std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
        time::SystemTime,
    };
    flat! {
        /// *This requires the `alloc` feature, which is enabled by default*
        IpAddr, Ipv4Addr, Ipv6Addr,
        SocketAddr, SocketAddrV4, SocketAddrV6,
        SystemTime
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use super::*;
    use ::alloc::string::String;
    flat! {
        // *This requires the `alloc` feature, which is enabled by default*
        String
    }
}
