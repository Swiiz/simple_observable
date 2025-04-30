use simple_observable::*;

// Automatically derives `Observable` for `Example`, `Default` for Changes<Example>
// and applies `PartialEq`, and `Eq` to `Observer<Example>` and `Changes<Example>`.
#[observable(Debug, PartialEq, Eq)]
pub struct Example {
    a: i32,
    b: i16,
}

fn main() {
    let mut obs: Observer<Example> = Default::default();
    let mut state = Example { a: 12, b: 52 };

    println!("{:?}", state.pull_changes(&mut obs));
    state.a += 1;
    println!("{:?}", state.pull_changes(&mut obs));
    state.b -= 2;
    state.a -= 1;
    println!("{:?}", state.pull_changes(&mut obs));
    let changes: Changes<Example> = state.pull_changes(&mut obs);
    assert!(changes == Changes::<Example> { a: 0, b: 0 });
}
