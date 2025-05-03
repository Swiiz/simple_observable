use std::ops::DerefMut;

use simple_observable::{
    Atomic, ChangeFilter, FilterObserver, IterObserver, MaybeObservable, Observable, Observer,
};

fn main() {
    let mut atom = Atomic::from(1);
    let mut obs: Observer<Atomic<i32>> = Default::default();

    println!("{:?}", atom.pull_changes(&mut obs));
    *atom += 1;
    println!("{:?}", atom.pull_changes(&mut obs));
    atom.deref_mut();
    println!("{:?}", atom.pull_changes(&mut obs));

    let mut atom_vec: ChangeFilter<Vec<Atomic<i32>>> =
        ChangeFilter(vec![Atomic::from(1), Atomic::from(2)]);
    let mut obs: FilterObserver<_> = Default::default();

    println!("{:?}", atom_vec.pull_changes(&mut obs));
    *atom_vec[0] += 1;
    println!("{:?}", atom_vec.pull_changes(&mut obs));
    atom_vec.push(Atomic::from(3));
    println!("{:?}", atom_vec.pull_changes(&mut obs));
}
