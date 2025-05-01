use simple_observable::*;

fn main() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
    let mut obs: AsIterObserver<Vec<i32>> = Default::default();
    // AsIterObserver<_> = Observer<ChangeIter<Vec<i32>>>

    println!("{:?}", vec.pull_changes(&mut obs));
    vec.push(6);
    vec.push(7);
    println!("{:?}", vec.pull_changes(&mut obs));
    vec.pop();
    println!("{:?}", vec.pull_changes(&mut obs));
}
