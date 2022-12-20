use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
fn add_with_return(a: u32, b: u32) -> u32 {
    a + b
}

#[test]
fn with_return_type() {
    let args = AddWithReturnArgs { a: 13, b: 37 };

    let original = add_with_return(args.a, args.b);
    let thunk = add_with_return_thunk(args);

    assert_eq!(original, thunk);
}
