//! ⚠️ *Should only be used if you know what you're doing*: Used for manual implementations

use crate::*;

#[macro_export]
macro_rules! impl_for {
    ($name:ident<$at:ident> => $($ty:ty);*) => {
        impl_for!(@gat $name<$at> => $($ty: $ty);*);
    };
    (@gat $name:ident<$at:ident> => $($ty:ty: $atv:ty);*) => { $( impl $name for $ty { type $at = $atv; } )* };
}

pub fn new_observer<'a, T: Observable>(inner: <T as Observable>::Observer<'a>) -> Observer<'a, T> {
    Observer { inner }
}

#[derive(Default)]
/// A struct representing the changes between the current and previous state of a Key-Value store.
pub struct MapChanges<'a, K, V: Observable + 'static> {
    pub changed: Box<[(&'a K, Changes<'a, V>)]>,
    pub removed: Box<[K]>,
}

pub trait TriviallyObservable: Default + Copy {
    type Detective: Default + From<Self>;
    fn delta(&self, observer: &Self::Detective) -> Self::Detective;
}
pub trait SubObservable: Default + Copy {
    type Detective: Default + From<Self> + std::ops::Sub<Output = Self::Detective> + Copy;
}
