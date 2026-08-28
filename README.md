# wombocombo

Generate every possible value of a struct or enum.

Derive `AllValues` on a type and you get `all_values()` (a `Vec`) and
`all_values_iter()` (a lazy, cloneable, `ExactSizeIterator`). For structs the
result is the full cartesian product of each field's possible values.

Field and variant types just need to implement `AllValuesIter`. It is already
implemented for `bool`, `std::cmp::Ordering`, `std::fmt::Alignment`, `Option<T>`,
`Result<T, E>`, and any type that derives `AllValues`.

## Install

```toml
[dependencies]
wombocombo = "0.1"
```

## Quick example

```rust
use wombocombo::AllValues;

#[derive(AllValues, Debug, Clone)]
enum Mode {
    Read,
    Write,
}

#[derive(AllValues, Debug, Clone)]
struct Config {
    mode: Mode,
    verbose: bool,
    retries: Option<bool>,
}

fn main() {
    let all = Config::all_values();

    // 2 modes * 2 bools * 3 Option<bool> values = 12 combinations
    assert_eq!(all.len(), 12);

    for Config { mode, verbose, retries } in &all {
        println!("{mode:?}, verbose={verbose}, retries={retries:?}");
    }

    // The iterator form keeps an exact size hint without allocating.
    assert_eq!(Config::all_values_iter().size_hint(), (12, Some(12)));
}
```

## Custom types

Implement `AllValuesIter` for a type the derive does not cover:

```rust
use wombocombo::{define_values_iter, AllValuesIter};

#[derive(Clone, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

define_values_iter!(
    Direction,
    [Direction::North, Direction::South, Direction::East, Direction::West]
);
```

## License

LGPL-3.0-or-later
