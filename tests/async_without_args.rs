use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
async fn async_add_without_args() -> u32 {
    5 + 5
}

#[test]
fn async_without_args() {
    async_std::task::block_on(async { assert_eq!(async_add_without_args_thunk().await, 10) });
}
