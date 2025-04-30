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

// Observe changes by subtraction
impl<T: std::ops::Sub<Output = T> + Default + Copy> Observable for T {
    type Observer = T;
    type Changes = T;
    fn pull_changes(&self, observer: &mut Self::Observer) -> Self::Changes {
        let changes = *self - *observer;
        *observer = *self;
        changes
    }
}
