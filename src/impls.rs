use std::{
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    Atomic, ChangeIter, Changes, Observable, ObservableAsIter, Observer, Untracked, impl_for,
    manual::{MapChanges, SubObservable, TriviallyObservable, new_observer},
};

impl<T: Observable> Default for Observer<'_, T> {
    fn default() -> Self {
        Observer {
            inner: Default::default(),
        }
    }
}

impl<T: TriviallyObservable + 'static> Observable for T {
    type Observer<'b> = T::Detective;
    type Changes<'b> = T::Detective;

    fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
        let changes = self.delta(&observer.inner);
        *observer = new_observer((*self).into());
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

    fn pull_changes(&self, _observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
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

    fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
        let (hasher, previous_hash) = &mut observer.inner;
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
            type Observer<'b> = ($(Observer<'b, $ty>,)*);
            type Changes<'b> = ($(Changes<'b, $ty>,)*);

            fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
                ($( self.$i.pull_changes(&mut observer.inner.$i), )*)
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

pub(crate) fn pull_iter_changes<'a, T: AsIter>(
    iter: &'a T,
    observer: &mut Vec<Observer<<T as AsIter>::Item>>,
) -> Box<[Changes<'a, <T as AsIter>::Item>]>
where
    <T as AsIter>::Item: Observable,
{
    let prev_len = observer.len();

    let changes: Box<_> = iter
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if i >= prev_len {
                observer.push(Default::default())
            }
            v.pull_changes(&mut observer[i])
        })
        .collect();
    if observer.len() > changes.len() {
        observer.truncate(changes.len());
    }
    changes
}

impl<T: 'static + AsIter> Observable for ChangeIter<T>
where
    <T as AsIter>::Item: Observable,
{
    type Observer<'a> = Vec<Observer<'a, <T as AsIter>::Item>>;
    type Changes<'a>
        = Box<[<<T as AsIter>::Item as Observable>::Changes<'a>]>
    where
        Self: 'a;

    fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
        pull_iter_changes(&self.0, &mut observer.inner)
    }
}

impl<T: AsIter + 'static> ObservableAsIter for T
where
    ChangeIter<T>: for<'a> Observable<Observer<'a> = Vec<Observer<'a, <T as AsIter>::Item>>>,
    <T as AsIter>::Item: Observable,
{
    fn pull_changes(
        &self,
        observer: &mut Observer<ChangeIter<Self>>,
    ) -> Box<[<<T as AsIter>::Item as Observable>::Changes<'_>]> {
        pull_iter_changes(self, &mut observer.inner)
    }
}

impl<K: 'static + Hash + Eq + Clone, V: Observable + 'static> Observable for HashMap<K, V> {
    type Observer<'a> = HashMap<K, Observer<'a, V>>;
    type Changes<'a> = MapChanges<'a, K, V>;

    fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Changes<'_, Self> {
        let changed: Box<_> = self
            .iter()
            .map(|(k, v)| {
                (
                    k,
                    v.pull_changes(observer.inner.entry(k.clone()).or_default()),
                )
            })
            .collect();

        let n_removed = observer.inner.len() - self.len();
        let mut removed = Vec::with_capacity(n_removed);

        if n_removed > 0 {
            // Items were removed from hashmap
            let to_remove: Box<_> = observer
                .inner
                .keys()
                .filter(|k| !self.contains_key(k))
                .cloned()
                .collect();
            for k in to_remove {
                observer.inner.remove(&k);
                removed.push(k);
            }
        }

        MapChanges {
            changed,
            removed: removed.into_boxed_slice(),
        }
    }
}

impl<K: Debug, V: Observable + Debug> Debug for MapChanges<'_, K, V>
where
    for<'a> <V as Observable>::Changes<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapChanges")
            .field("changed", &self.changed)
            .field("removed", &self.removed)
            .finish()
    }
}
