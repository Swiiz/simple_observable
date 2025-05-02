#![doc = include_str!("../README.md")]

mod impls;
pub mod manual;

use impls::AsIter;
pub use proc_macro::*;

#[rustfmt::skip]
pub struct Observer<'a, T: Observable + ?Sized> { pub inner: <T as Observable>::Observer<'a> }
pub type Changes<'a, T> = <T as Observable>::Changes<'a>;

pub type AsIterObserver<'a, T> = Observer<'a, ChangeIter<T>>;

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

/// A wrapper that treats the inner value as untracked and always reports it as changed.
///
/// Use this when you don’t need change detection and just want to carry the current value each time.
pub struct Untracked<T>(pub T);

/// A wrapper that observes `T` as a single atomic value and only reports changes if the value actually differs.
///
/// If the value is equal to the previously observed one, no change is reported.
///
/// This is useful for types where partial diffing isn’t possible, but you still want to avoid redundant updates.
pub struct Atomic<T, H = std::hash::DefaultHasher>(pub T, std::marker::PhantomData<H>);

/// A wrapper that treats the inner value as an iterable and reports changes for each element.
///
/// Use this when you need change detection for each element of an iterable.
///
/// (Wrapping the type in `ChangeIter<T>` is optional if you are willing to use `AsIterObserver`.)
pub struct ChangeIter<T>(pub T);

/// A trait for types that can be observed as an iterable.
///
/// Used to avoid having to wrap the iterable in `ChangeIter<T>` when using `Observable`.
pub trait ObservableAsIter: AsIter + Sized
where
    ChangeIter<Self>: Observable,
    <Self as AsIter>::Item: Observable,
{
    fn pull_changes(
        &self,
        observer: &mut Observer<ChangeIter<Self>>,
    ) -> Vec<<<Self as AsIter>::Item as Observable>::Changes<'_>>;
}
