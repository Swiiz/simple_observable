use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    Atomic, ChangeIter, Observable, ObservableAsIter, Untracked, impl_for,
    manual::{SubObservable, TriviallyObservable},
};

impl<T: TriviallyObservable + 'static> Observable for T {
    type Observer<'b> = T::Detective;
    type Changes<'b> = T::Detective;

    fn pull_changes(&self, observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
        let changes = self.delta(observer);
        *observer = (*self).into();
        changes
    }
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

impl<'a, T: 'static> Observable for Untracked<T> {
    type Observer<'b> = ();
    type Changes<'b> = &'b T;

    fn pull_changes(&self, _observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
        &self
    }
}

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

impl<'a, T: 'static + Hash, H: Hasher + Default + 'static> Observable for Atomic<T, H> {
    type Observer<'b> = (H, u64);
    type Changes<'b> = Option<&'b T>;

    fn pull_changes(&self, observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
        let (hasher, previous_hash) = observer;
        self.0.hash(hasher);
        let current_hash = hasher.finish();
        (*previous_hash != current_hash).then(|| {
            *previous_hash = current_hash;
            &self as &T
        })
    }
}

macro_rules! impl_obs_for_tuples {
    ($($ty:ident $i:tt)*) => {
        impl<'a, $($ty: Observable + 'static),*> Observable for ($($ty,)*) {
            type Observer<'b> = ($(<$ty as Observable>::Observer<'b>,)*);
            type Changes<'b> = ($(<$ty as Observable>::Changes<'b>,)*);

            fn pull_changes(&self, observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
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

impl<T: 'static + AsIter> Observable for ChangeIter<T>
where
    <T as AsIter>::Item: Observable,
{
    type Observer<'a> = Vec<<<T as AsIter>::Item as Observable>::Observer<'a>>;
    type Changes<'a>
        = Vec<<<T as AsIter>::Item as Observable>::Changes<'a>>
    where
        Self: 'a;

    fn pull_changes(&self, observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
        pull_iter_changes(&self.0, observer)
    }
}

pub(crate) fn pull_iter_changes<'a, T: AsIter>(
    iter: &'a T,
    observer: &mut Vec<<<T as AsIter>::Item as Observable>::Observer<'_>>,
) -> Vec<<<T as AsIter>::Item as Observable>::Changes<'a>>
where
    ChangeIter<T>: Observable,
    <T as AsIter>::Item: Observable,
{
    let prev_len = observer.len();
    iter.iter()
        .enumerate()
        .map(|(i, v)| {
            if i >= prev_len {
                observer.push(Default::default())
            }
            v.pull_changes(&mut observer[i])
        })
        .collect()
}

impl<T> Deref for ChangeIter<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ChangeIter<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for ChangeIter<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

pub trait AsIter {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

impl<T: IntoIterator> AsIter for T
where
    for<'a> &'a T: IntoIterator<Item = &'a <T as IntoIterator>::Item>,
{
    type Item = <T as IntoIterator>::Item;
    type Iter<'a>
        = <&'a T as IntoIterator>::IntoIter
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.into_iter()
    }
}

impl<T: AsIter + 'static> ObservableAsIter for T
where
    <T as AsIter>::Item: Observable,
{
    fn pull_changes<'a>(
        &'a self,
        observer: &mut Vec<<<Self as AsIter>::Item as Observable>::Observer<'_>>,
    ) -> Vec<<<Self as AsIter>::Item as Observable>::Changes<'a>>
    where
        <Self as AsIter>::Item: Observable,
    {
        pull_iter_changes(self, observer)
    }
}
