use serde::Deserialize;
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
