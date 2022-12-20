use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
async fn async_add_with_return_type(a: u32, b: u32) -> u32 {
    a + b
}

#[test]
fn async_with_return_type() {
    let args = AsyncAddWithReturnTypeArgs { a: 13, b: 37 };

    async_std::task::block_on(async {
        let original = async_add_with_return_type(args.a, args.b).await;
        let thunk = async_add_with_return_type_thunk(args).await;

        assert_eq!(original, thunk);
    });
}
