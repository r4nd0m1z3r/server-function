use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
fn add_without_args() -> u32 {
    5 + 5
}

#[test]
fn without_args() {
    assert_eq!(add_without_args_thunk(), 10)
}
