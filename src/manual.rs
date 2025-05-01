#[macro_export]
macro_rules! impl_for {
    ($name:ident<$at:ident> => $($ty:ty);*) => {
        impl_for!(@gat $name<$at> => $($ty: $ty);*);
    };
    (@gat $name:ident<$at:ident> => $($ty:ty: $atv:ty);*) => { $( impl $name for $ty { type $at = $atv; } )* };
}

pub trait TriviallyObservable: Default + Copy {
    type Detective: Default + From<Self>;
    fn delta(&self, observer: &Self::Detective) -> Self::Detective;
}
pub trait SubObservable: Default + Copy {
    type Detective: Default + From<Self> + std::ops::Sub<Output = Self::Detective> + Copy;
}
