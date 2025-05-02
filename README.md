# 🔭 simple_observable 🦀 ([Docs](https://swiiz.github.io/simple_observable))

A simple Rust crate to track and observe changes in the state of your types.

## Features
- Monitor changes from the perspective of multiple observers.
- Automatically derives the `Observable` trait for structs with the `#[observable]` attribute macro.
- Tracks and computes changes between the current state and the previous state of your struct.
- Supports custom derives (e.g., `Debug`, `PartialEq`, `Eq`) for the generated `Changes` types.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
simple_observable = "0.1"
```

### Deriving `Observable` for a Struct

Use the `#[observable]` attribute to derive the `Observable` trait for your struct:

```rust
use simple_observable::*;

// Automatically derives `Observable` for `Example`, `Default` for Observer<Example>
// and applies `Debug`, `PartialEq`, and `Eq` to `Changes<Example>`.
#[observable(Debug, PartialEq, Eq)]
pub struct Example {
    a: i32,
    b: i16,
}

fn main() {
    let mut observer: Observer<Example> = Default::default();
    let mut state = Example { a: 12, b: 52 };

    println!("{:?}", state.pull_changes(&mut observer));
    state.a += 1;
    println!("{:?}", state.pull_changes(&mut observer));
    state.b -= 2;
    state.a -= 1;
    println!("{:?}", state.pull_changes(&mut observer));
}
```

### How It Works:
1. `#[observable]` generates two types: `Observer<YourStruct>` and `Changes<YourStruct>`.
2. The `Observer` tracks the changes from its perspective, multiple observers can track the same state independently.
3. The `Changes` holds the computed differences between the current and previous state.
4. `pull_changes` method calculates and returns the changes while updating the observer.

### Todos:
- Built-in types with optimized implementations for observing.
- Support for more types.