# What is this

This is a macro for RPC programming needs which generates a structure and a thunk for function in a way that allows calling that function with a struct argument that supports ```serde::Deserialize```

## How to use

1. Mark function with ```#[server_function]``` attribute

    ```rust
    #[server_function]
    fn add(a: u32, b: u32) -> u32 {
        a + b
    }
    ```

2. This will generate ```AddArgs``` struct matching your function arguments that looks like this

    ```rust
    struct AddArgs {
        a: u32,
        b: u32
    }
    ```

    and a function named ```add_thunk``` with ```AddArgs``` struct as 1 argument
3. Now you can call it

    ```rust
    // will return 10, same as original function
    add_thunk(AddArgs { a: 5, b: 5 })
    ```
