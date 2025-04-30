#![doc = include_str!("../README.md")]

pub use proc_macro::*;

pub type Observer<T> = <T as Observable>::Observer;
pub type Changes<T> = <T as Observable>::Changes;

/// A trait for types that can be observed, allowing for change detection.
///
/// This trait can be manually implemented or derived using the [`#[observable]`](crate::observable) attribute.
pub trait Observable {
    /// The type used to observe changes.
    type Observer: Default;

    /// The type representing the changes between states
    type Changes: Default;

    /// Pulls changes since the last observed state and updates the observer.
    ///
    /// Returns the changes since the last observed changes using this observer.
    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes;
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

impl<T: TriviallyObservable> Observable for T {
    type Observer = T::Detective;
    type Changes = T::Detective;

    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes {
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
