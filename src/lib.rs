#![feature(iter_array_chunks)]
#![feature(let_chains)]

use convert_case::{Case, Casing};
use proc_macro::TokenStream as TokenStreamV1;
use proc_macro2::{Delimiter, Ident, TokenStream as TokenStreamV2, TokenTree};
use quote::{format_ident, quote};

fn get_fn_name(mut tokens_iter: impl Iterator<Item = TokenTree>) -> Option<(Ident, bool)> {
    let mut is_next_token_fn_name = false;
    let mut is_async = false;
    let fn_name = tokens_iter.find(|token_tree| {
        if is_next_token_fn_name {
            return true;
        }
        if let TokenTree::Ident(ident) = token_tree {
            if ident == "async" {
                is_async = true;
            }
            if ident == "fn" {
                is_next_token_fn_name = true;
                return false;
            } else {
                return false;
            }
        }
        false
    })?;

    if let TokenTree::Ident(ident) = fn_name {
        Some((ident, is_async))
    } else {
        None
    }
}

fn get_fn_return_type(tokens_iter: impl Iterator<Item = TokenTree>) -> Option<Ident> {
    let mut is_next_token_return_type = false;
    let return_type = tokens_iter.array_chunks::<2>().find(|[token1, token2]| {
        if is_next_token_return_type {
            return true;
        }
        if let TokenTree::Punct(punct1) = token1 && let TokenTree::Punct(punct2) = token2 {
            if punct1.as_char() == '-' && punct2.as_char() == '>' {
                is_next_token_return_type = true;
                return false;
            } else {
                return false;
            }
        }
        false
    })?;

    if let TokenTree::Ident(return_type) = return_type[0].clone() {
        Some(return_type)
    } else {
        None
    }
}

fn generate_struct(
    fn_name: &Ident,
    mut tokens_iter: impl Iterator<Item = TokenTree>,
) -> Option<(TokenStreamV2, Ident)> {
    let struct_name = format_ident!("{}Args", fn_name.to_string().to_case(Case::Pascal));

    let fn_args_tokens = {
        let fn_args_group = tokens_iter.find(|token_tree| {
            if let TokenTree::Group(group) = token_tree {
                group.delimiter() == Delimiter::Parenthesis
            } else {
                false
            }
        })?;
        if let TokenTree::Group(group) = fn_args_group {
            group.stream()
        } else {
            return None;
        }
    };

    Some((
        quote! {
            #[derive(Deserialize, Debug)]
            struct #struct_name {
                #fn_args_tokens
            }
        },
        struct_name,
    ))
}

fn generate_thunk(
    is_async: bool,
    fn_name: &Ident,
    struct_name: &Ident,
    return_type: Option<Ident>,
    mut tokens_iter: impl Iterator<Item = TokenTree>,
) -> Option<TokenStreamV2> {
    let thunk_name = format_ident!("{}_thunk", fn_name);

    let struct_tokens = {
        let struct_group = tokens_iter.find(|token_tree| {
            if let TokenTree::Group(group) = token_tree {
                group.delimiter() == Delimiter::Brace
            } else {
                false
            }
        })?;
        if let TokenTree::Group(group) = struct_group {
            group.stream()
        } else {
            return None;
        }
    };

    let mut should_filter_next = false;
    let unwrap_stream = struct_tokens
        .into_iter()
        .filter(|token_tree| {
            if should_filter_next {
                should_filter_next = false;
                return false;
            }
            if let TokenTree::Punct(punct) = token_tree {
                if punct.as_char() == ':' {
                    should_filter_next = true;
                    return false;
                } else {
                    return true;
                }
            }
            true
        })
        .collect::<TokenStreamV2>();

    if let Some(return_type) = return_type {
        if is_async {
            Some(quote! {
                async fn #thunk_name (args: #struct_name) -> #return_type {
                    let #struct_name { #unwrap_stream } = args;
                    #fn_name(#unwrap_stream).await
                }
            })
        } else {
            Some(quote! {
                fn #thunk_name (args: #struct_name) -> #return_type {
                    let #struct_name { #unwrap_stream } = args;
                    #fn_name(#unwrap_stream)
                }
            })
        }
    } else if is_async {
        Some(quote! {
            async fn #thunk_name (args: #struct_name) {
                let #struct_name { #unwrap_stream } = args;
                #fn_name(#unwrap_stream).await;
            }
        })
    } else {
        Some(quote! {
            fn #thunk_name (args: #struct_name) {
                let #struct_name { #unwrap_stream } = args;
                #fn_name(#unwrap_stream);
            }
        })
    }
}

#[proc_macro_attribute]
pub fn server_function(_attr: TokenStreamV1, item: TokenStreamV1) -> TokenStreamV1 {
    let item = Into::<TokenStreamV2>::into(item);
    let mut item_iter = item.clone().into_iter();

    let (fn_name, is_async) = get_fn_name(&mut item_iter).expect("Failed to get function name!");
    let (args_struct, args_struct_name) = generate_struct(&fn_name, &mut item_iter)
        .expect("Failed to generate function arguments struct!");
    let return_type = get_fn_return_type(&mut item_iter);
    let thunk = generate_thunk(
        is_async,
        &fn_name,
        &args_struct_name,
        return_type,
        args_struct.clone().into_iter(),
    )
    .expect("Failed to generate function thunk!");

    quote! {
        #args_struct
        #thunk

        #item
    }
    .into()
}
