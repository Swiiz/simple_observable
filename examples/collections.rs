use std::collections::HashMap;

use simple_observable::*;

#[derive(Default)]
#[observable(Debug)]
pub struct Example {
    // standard HashMap is supported but may have some overhead for large maps
    map: HashMap<String, i16>,
}

fn main() {
    iter_example();

    let (mut state, mut obs): (Example, Observer<Example>) = Default::default();

    println!("{:?}", state.pull_changes(&mut obs));
    state.map.insert("a".to_string(), 1);
    println!("{:?}", state.pull_changes(&mut obs));
    state.map.remove("a");
    state.map.insert("b".to_string(), 2);
    println!("{:?}", state.pull_changes(&mut obs));
    state.map.remove("c");
    println!("{:?}", state.pull_changes(&mut obs));
}

fn iter_example() {
    // Simple iterators can use ChangesIter(_) wrapper (such as vec, boxed slice etc...)
    // pull_changes can still be called without using the wrapper by using the ObservableAsIter trait
    // For compatibility with the #[observable] macro, use the wrapper (which implements Observable trait)
    let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
    let mut obs: AsIterObserver<Vec<i32>> = Default::default();

    println!("{:?}", vec.pull_changes(&mut obs));
    vec.push(6);
    vec.push(7);
    println!("{:?}", vec.pull_changes(&mut obs));
    vec.pop();
    println!("{:?}", vec.pull_changes(&mut obs));
    vec.push(8);
    println!("{:?}", vec.pull_changes(&mut obs));
}
