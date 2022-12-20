#![cfg(feature = "messagepack")]

use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
fn add_with_return(a: u32, b: u32) -> u32 {
    a + b
}

#[test]
fn messagepack() {
    let bytes = rmp_serde::to_vec(&AddWithReturnArgs { a: 5, b: 5 }).unwrap();
    let messagepack_thunk_result = add_with_return_messagepack_thunk(&bytes);

    assert_eq!(messagepack_thunk_result, 10);
}
