use from_remote_derive::FromRemote;

#[derive(FromRemote)]
#[from_remote = "Bar"]
struct Foo {
    bar: u64,
    fizz: Vec<String>,
}

struct Bar {
    bar: u64,
    fizz: Vec<String>,
}

#[derive(FromRemote)]
#[from_remote = "Buzz"]
enum Fizz {
    A(u64, Option<u64>),
    B(String),
    C(Foo),
    D {
        x: u16,
        y: u32,
    },
}

enum Buzz {
    A(u64, Option<u64>),
    B(String),
    C(Bar),
    D {
        x: u16,
        y: u32,
    },
}

fn main() {}
