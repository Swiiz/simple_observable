#![doc = include_str!("../README.md")]

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use proc_macro::*;

pub type Observer<T> = <T as Observable>::Observer;
pub type Changes<'a, T> = <T as Observable>::Changes<'a>;

/// A trait for types that can be observed, allowing for change detection.
///
/// This trait can be manually implemented or derived using the [`#[observable]`](crate::observable) attribute.
pub trait Observable {
    /// The type used to observe changes.
    type Observer: Default;

    /// The type representing the changes between states
    type Changes<'a>
    where
        Self: 'a;

    /// Pulls changes since the last observed state and updates the observer.
    ///
    /// Returns the changes since the last observed changes using this observer.
    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes<'_>;
}

/// A wrapper that treats the inner value as untracked and always reports it as changed.
///
/// Use this when you don’t need change detection and just want to carry the current value each time.
pub struct Untracked<T>(pub T);

impl<T> Deref for Untracked<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Untracked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Untracked<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: 'static> Observable for Untracked<T> {
    type Observer = ();
    type Changes<'a> = &'a T;

    fn pull_changes(&self, _observer: &mut Self::Observer) -> Self::Changes<'_> {
        &self
    }
}

/// A wrapper that observes `T` as a single atomic value and only reports changes if the value actually differs.
///
/// If the value is equal to the previously observed one, no change is reported.
///
/// This is useful for types where partial diffing isn’t possible, but you still want to avoid redundant updates.
pub struct Atomic<T, H = DefaultHasher>(pub T, PhantomData<H>);

impl<T, H> Deref for Atomic<T, H> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, H> DerefMut for Atomic<T, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, H> From<T> for Atomic<T, H> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: 'static + Hash, H: Hasher + Default + 'static> Observable for Atomic<T, H> {
    type Observer = (H, u64);
    type Changes<'a> = Option<&'a T>;

    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes<'_> {
        let (hasher, previous_hash) = observer;
        self.0.hash(hasher);
        let current_hash = hasher.finish();
        (*previous_hash != current_hash).then(|| {
            *previous_hash = current_hash;
            &self as &T
        })
    }
}

pub trait TriviallyObservable: Default + Copy {
    type Detective: Default + From<Self>;
    fn delta(&self, observer: &Self::Detective) -> Self::Detective;
}

#[macro_export]
macro_rules! impl_for {
        ($name:ident<$at:ident> => $($ty:ty);*) => {
            impl_for!(@gat $name<$at> => $($ty: $ty);*);
        };
        (@gat $name:ident<$at:ident> => $($ty:ty: $atv:ty);*) => { $( impl $name for $ty { type $at = $atv; } )* };
    }

impl<T: TriviallyObservable + 'static> Observable for T {
    type Observer = T::Detective;
    type Changes<'a> = T::Detective;

    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes<'_> {
        let changes = self.delta(observer);
        *observer = (*self).into();
        changes
    }
}

pub trait SubObservable: Default + Copy {
    type Detective: Default + From<Self> + std::ops::Sub<Output = Self::Detective> + Copy;
}
impl<T: SubObservable> TriviallyObservable for T {
    type Detective = <T as SubObservable>::Detective;

    fn delta(&self, observer: &Self::Detective) -> Self::Detective {
        Into::<Self::Detective>::into(*self) - *observer
    }
}

impl TriviallyObservable for bool {
    type Detective = bool;
    fn delta(&self, observer: &Self) -> Self::Detective {
        let changes = *self != *observer;
        changes
    }
}

impl_for!(SubObservable<Detective> => i8; i16; i32; i64; i128; f32; f64);
impl_for!(@gat SubObservable<Detective> => u8: i16; u16: i32; u32: i64; u64: i128);

macro_rules! impl_obs_for_tuples {
    ($($ty:ident $i:tt)*) => {
        impl<$($ty: Observable + 'static),*> Observable for ($($ty,)*) {
            type Observer = ($(<$ty as Observable>::Observer,)*);
            type Changes<'a> = ($(<$ty as Observable>::Changes<'a>,)*);

            fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes<'_> {
                ($( self.$i.pull_changes(&mut observer.$i), )*)
            }
        }
    };
}

impl_obs_for_tuples!(A 0);
impl_obs_for_tuples!(A 0 B 1);
impl_obs_for_tuples!(A 0 B 1 C 2);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3 E 4);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3 E 4 F 5);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3 E 4 F 5 G 6);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7);
impl_obs_for_tuples!(A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8);

//TODO: implement for collections
/*
const _: () = {
    #[derive(Default)]
    pub struct VecObserver;
    pub enum VecOp {}

    impl<T> Observable for Vec<T> {
        type Observer = VecObserver;
        type Changes = Vec<VecOp>;

        fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes {
            todo!()
        }
    }
};
 */
