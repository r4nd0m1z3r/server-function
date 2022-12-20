#![feature(iter_array_chunks)]
#![feature(let_chains)]

use convert_case::{Case, Casing};
use proc_macro::TokenStream as TokenStreamV1;
use proc_macro2::{Delimiter, Ident, TokenStream as TokenStreamV2, TokenTree};
use quote::{format_ident, quote, TokenStreamExt};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ThunkType {
    Default,
    MessagePack,
}

#[derive(Debug)]
struct FnData {
    is_async: bool,
    name: Ident,
    return_type: Option<Ident>,
}
impl FnData {
    fn get_fn_name(token_stream: TokenStreamV2) -> Result<(Ident, bool), ()> {
        let mut tokens_iter = token_stream.into_iter();

        let mut is_next_token_fn_name = false;
        let mut is_async = false;
        let fn_name = tokens_iter
            .find(|token_tree| {
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
            })
            .ok_or(())?;

        if let TokenTree::Ident(ident) = fn_name {
            Ok((ident, is_async))
        } else {
            Err(())
        }
    }

    fn get_fn_return_type(token_stream: TokenStreamV2) -> Option<Ident> {
        let tokens_iter = token_stream.into_iter();

        let mut return_type_token_index = None;
        let mut is_next_token_return_type = false;
        let return_type = tokens_iter.array_chunks::<2>().find(|[token1, token2]| {
            if is_next_token_return_type {
                return true;
            }
            if let TokenTree::Punct(punct1) = token1 && let TokenTree::Punct(punct2) = token2 {
                let p1_char = punct1.as_char();
                let p2_char = punct2.as_char();

                if p1_char == '-' && p2_char == '>' {
                    is_next_token_return_type = true;
                    return_type_token_index = Some(0);
                    return false;
                } else {
                    return false;
                }
            }
            else if let TokenTree::Punct(punct) = token1 && let TokenTree::Ident(_) = token2 {
                let p_char = punct.as_char();

                if p_char == '>' {
                    return_type_token_index = Some(1);
                    return true;
                }
            }
            false
        })?;

        if let TokenTree::Ident(return_type) = return_type[return_type_token_index.unwrap()].clone()
        {
            Some(return_type)
        } else {
            None
        }
    }

    fn from_token_stream(token_stream: TokenStreamV2) -> Option<Self> {
        let return_type = Self::get_fn_return_type(token_stream.clone());
        let (fn_name, is_async) = Self::get_fn_name(token_stream).ok()?;

        Some(Self {
            is_async,
            name: fn_name,
            return_type,
        })
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
            #[derive(Serialize, Deserialize, Debug)]
            struct #struct_name {
                #fn_args_tokens
            }
        },
        struct_name,
    ))
}

fn get_struct_field_names(tokens_iter: impl Iterator<Item = TokenTree>) -> Option<TokenStreamV2> {
    let mut should_filter_next = false;

    let variable_names_tokens = tokens_iter
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

    if variable_names_tokens.is_empty() {
        None
    } else {
        Some(variable_names_tokens)
    }
}

fn get_struct_fields(mut tokens_iter: impl Iterator<Item = TokenTree>) -> Option<TokenStreamV2> {
    let struct_fields_group = tokens_iter.find(|token_tree| {
        if let TokenTree::Group(group) = token_tree {
            group.delimiter() == Delimiter::Brace
        } else {
            false
        }
    })?;

    if let TokenTree::Group(group) = struct_fields_group {
        let stream = group.stream();

        if stream.is_empty() {
            None
        } else {
            Some(stream)
        }
    } else {
        None
    }
}

fn generate_thunk(
    fn_data: &FnData,
    struct_name: &Ident,
    tokens_iter: impl Iterator<Item = TokenTree>,
    thunk_type: ThunkType,
) -> Option<TokenStreamV2> {
    let FnData {
        is_async,
        name,
        return_type,
    } = fn_data;

    let thunk_name = match thunk_type {
        ThunkType::Default => format_ident!("{}_thunk", name),
        ThunkType::MessagePack => format_ident!("{}_messagepack_thunk", name),
    };

    let struct_fields_tokens = get_struct_fields(tokens_iter);

    let variable_names_tokens = if struct_fields_tokens.is_some() {
        get_struct_field_names(struct_fields_tokens?.into_iter())
    } else {
        None
    };

    let fn_prefix = if *is_async {
        quote!(async fn)
    } else {
        quote!(fn)
    };

    let args_token_stream = if variable_names_tokens.is_none() {
        quote!(())
    } else {
        match thunk_type {
            ThunkType::Default => quote!((args: #struct_name)),
            ThunkType::MessagePack => quote!((bytes: &[u8])),
        }
    };

    let return_type_stream = if return_type.is_none() {
        quote!()
    } else {
        quote!(-> #return_type)
    };

    let struct_unwrap_tokens = if variable_names_tokens.is_none() {
        quote!()
    } else {
        quote!(let #struct_name { #variable_names_tokens } = args;)
    };

    let mut call_token_stream = if *is_async {
        quote!(#name(#variable_names_tokens).await)
    } else {
        quote!(#name(#variable_names_tokens))
    };
    if return_type.is_none() {
        call_token_stream.append_all(quote!(;));
    }

    match thunk_type {
        ThunkType::Default => Some(quote! {
            #fn_prefix #thunk_name #args_token_stream #return_type_stream {
                #struct_unwrap_tokens
                #call_token_stream
            }
        }),
        ThunkType::MessagePack => {
            if variable_names_tokens.is_some() {
                Some(quote! {
                    #fn_prefix #thunk_name #args_token_stream #return_type_stream {
                        let args = rmp_serde::from_slice(bytes).unwrap();
                        #struct_unwrap_tokens
                        #call_token_stream
                    }
                })
            } else {
                None
            }
        }
    }
}

#[proc_macro_attribute]
pub fn server_function(_attr: TokenStreamV1, item: TokenStreamV1) -> TokenStreamV1 {
    let item = Into::<TokenStreamV2>::into(item);
    let mut item_iter = item.clone().into_iter();

    let fn_data =
        FnData::from_token_stream(item.clone()).expect("Failed to extract function data!");
    let (args_struct, args_struct_name) = generate_struct(&fn_data.name, &mut item_iter)
        .expect("Failed to generate function arguments struct!");
    let thunk = generate_thunk(
        &fn_data,
        &args_struct_name,
        args_struct.clone().into_iter(),
        ThunkType::Default,
    )
    .expect("Failed to generate function thunk!");

    #[cfg(not(feature = "messagepack"))]
    return quote! {
        #args_struct
        #thunk

        #item
    }
    .into();

    #[cfg(feature = "messagepack")]
    let messagepack_thunk = generate_thunk(
        &fn_data,
        &args_struct_name,
        args_struct.clone().into_iter(),
        ThunkType::MessagePack,
    );
    #[cfg(feature = "messagepack")]
    quote! {
        #args_struct
        #thunk
        #messagepack_thunk

        #item
    }
    .into()
}
