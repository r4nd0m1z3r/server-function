use serde::Deserialize;
use server_function::server_function;

#[server_function]
fn add_with_return(a: u32, b: u32) -> u32 {
    a + b
}

#[server_function]
fn add_without_return(a: u32, b: u32) {
    println!("{}", a + b);
}

#[test]
fn with_return_type() {
    let args = AddWithReturnArgs { a: 13, b: 37 };

    let original = add_with_return(args.a, args.b);
    let thunk = add_with_return_thunk(args);

    assert_eq!(original, thunk);
}

#[test]
fn without_return_type() {
    let args = AddWithoutReturnArgs { a: 13, b: 37 };

    add_without_return(args.a, args.b);
    add_without_return_thunk(args);
}
