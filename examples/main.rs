use std::collections::{HashMap, BTreeMap};

use from_remote_derive::FromRemote;

#[derive(FromRemote)]
#[from_remote("Bar", "Pook")]
struct Foo {
    bar: BTreeMap<u64, u64>,
    fizz: Vec<String>,
}

struct Bar {
    bar: BTreeMap<u64, u64>,
    fizz: Vec<String>,
}

struct Pook {
    bar: BTreeMap<u64, u64>,
    fizz: Vec<String>,
}

#[derive(FromRemote)]
#[from_remote("Buzz")]
enum Fizz {
    A(u64, Option<u64>),
    B(HashMap<String, Foo>),
    C(Foo),
    D {
        x: u16,
        y: u32,
    },
}

enum Buzz {
    A(u64, Option<u64>),
    B(HashMap<String, Bar>),
    C(Bar),
    D {
        x: u16,
        y: u32,
    },
}

fn main() {}
