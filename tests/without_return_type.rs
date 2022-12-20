use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
fn add_without_return(a: u32, b: u32) {
    println!("{}", a + b);
}

#[test]
fn without_return_type() {
    let args = AddWithoutReturnArgs { a: 13, b: 37 };

    add_without_return(args.a, args.b);
    add_without_return_thunk(args);
}
