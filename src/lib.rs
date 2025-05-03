#![doc = include_str!("../README.md")]

mod impls;
pub mod manual;
use manual::*;

pub use proc_macro::*;

/// Wrapper type for an observer.
pub struct Observer<'a, T: Observable + ?Sized> {
    pub inner: <T as Observable>::Observer<'a>,
}
pub type Changes<'a, T> = <T as Observable>::Changes<'a>;

pub type IterObserver<'a, T> = Observer<'a, ChangeIter<T>>;
pub type FilterObserver<'a, T> = Observer<'a, ChangeFilter<T>>;

/// A trait for types that can be observed, allowing for change detection.
///
/// This trait can be manually implemented or derived using the [`#[observable]`](crate::observable) attribute.
pub trait Observable {
    /// The type used to observe changes.
    type Observer<'b>: Default;

    /// The type representing the changes between states
    type Changes<'b>
    where
        Self: 'b;

    /// Pulls changes since the last observed state and updates the observer.
    ///
    /// Returns the changes since the last observed changes using this observer.
    fn pull_changes(&self, observer: &mut Observer<'_, Self>) -> Self::Changes<'_>;
}

/// A trait for types that can be observed, but yielding `Option<_>` as `Changes`.
pub trait MaybeObservable: Observable {
    type ChangesInner<'a>
    where
        Self: 'a;
    fn try_pull_changes(&self, observer: &mut Observer<'_, Self>)
    -> Option<Self::ChangesInner<'_>>;
}

/// A wrapper that treats the inner value as untracked and always reports it as changed.
///
/// Use this when you don’t need change detection and just want to carry the current value each time.
#[derive(Default)]
pub struct Untracked<T>(pub T);

/// A wrapper that observes `T` as a single atomic value and only reports changes if the value actually differs.
///
/// If the value was mutably accessed since the last observation, it will be reported as changed.
/// ### ⚠️ Interior mutability is not supported. (use `AtomicHash` instead)
///
/// This is useful for types where partial diffing isn’t possible, but you still want to avoid redundant updates.
#[derive(Default, Debug)]
pub struct Atomic<T, N = u16> {
    inner: T,
    counter: N,
}

impl<T, N: std::ops::AddAssign + From<u8>> Atomic<T, N> {
    pub fn set_changed(&mut self) {
        self.counter += N::from(1);
    }
}

/// A wrapper that treats the inner value as a hash and reports changes if the hash changes.
/// Similar to `Atomic<T, N>` but for hashes.
///
/// ### ✅ Useful for types with interior mutability
#[derive(Default, Debug)]
pub struct AtomicHash<T, H = std::hash::DefaultHasher>(pub T, std::marker::PhantomData<H>);

/// A wrapper that treats the inner value as an iterable and reports changes for each element.
///
/// Use this when you need change detection for each element of an iterable.
///
/// (Wrapping the type in `ChangeIter<T>` is optional if you are willing to use `AsIterObserver`.)
#[derive(Default, Debug)]
pub struct ChangeIter<T>(pub T);

/// A wrapper that treats the inner value as an iterable over `MaybeObservable`
/// and only reports changes for elements that actually changed.
#[derive(Default, Debug)]
pub struct ChangeFilter<T>(pub T);

/// A trait for types that can be observed as an iterable.
///
/// Used to avoid having to wrap the iterable in `ChangeIter<T>`.
pub trait ObservableAsIter: AsIter + Sized
where
    ChangeIter<Self>: Observable,
    <Self as AsIter>::Item: Observable,
{
    fn iter_changes(&self, observer: &mut Observer<ChangeIter<Self>>)
    -> ChangeIterReturn<'_, Self>;
}

/// A trait for types that can be observed as an iterable over `MaybeObservable`
///
/// Used to avoid having to wrap the iterable in `ChangeFilter<T>`.
pub trait ObservableAsFilter: AsIter + Sized
where
    ChangeFilter<Self>: Observable,
    <Self as AsIter>::Item: MaybeObservable,
{
    fn filter_changes(
        &self,
        observer: &mut Observer<ChangeFilter<Self>>,
    ) -> ChangeFilterReturn<'_, Self>;
}

//TODO: Add custom type/impl for ChangeFilter<Vec<Atomic<_>>> allowing for filtered changes iter.
