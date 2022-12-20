use serde::{Deserialize, Serialize};
use server_function::server_function;

#[server_function]
async fn async_add_without_return_type(a: u32, b: u32) {
    println!("{}", a + b);
}

#[test]
fn async_without_return_type() {
    let args = AsyncAddWithoutReturnTypeArgs { a: 13, b: 37 };

    async_std::task::block_on(async {
        async_add_without_return_type(args.a, args.b).await;
        async_add_without_return_type_thunk(args).await;
    });
}
