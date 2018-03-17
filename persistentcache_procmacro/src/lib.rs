// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Procedural macro to persistently cache functions.
//!
//! See the documentation of
//! [persistentcache](https://stefan-k.github.io/persistentcache-rs/persistentcache) for details.
//!
//! I would not have managed to write this code without the ideas that I shamelessly stole from
//! [accel](https://github.com/termoshtt/accel/).
#![feature(proc_macro)]
#![recursion_limit = "256"]

#[macro_use]
extern crate futures_await_quote as quote;
extern crate futures_await_syn as syn;
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::*;

#[derive(Debug)]
struct Function {
    attrs: Vec<syn::Attribute>,
    ident: Ident,
    vis: Visibility,
    block: Box<Block>,
    unsafety: Unsafety,
    inputs: delimited::Delimited<FnArg, tokens::Comma>,
    output: FunctionRetTy,
    fn_token: tokens::Fn_,
}

impl Function {
    fn parse(func: TokenStream) -> Self {
        let Item { node, attrs } = syn::parse(func.clone()).unwrap();
        let ItemFn {
            ident,
            vis,
            block,
            decl,
            unsafety,
            ..
        } = match node {
            ItemKind::Fn(item) => item,
            _ => unreachable!(),
        };
        let FnDecl {
            inputs,
            output,
            fn_token,
            ..
        } = { *decl };
        Function {
            attrs,
            ident,
            vis,
            block,
            unsafety,
            inputs,
            output,
            fn_token,
        }
    }
}

#[proc_macro_attribute]
pub fn persistent_cache(_attr: TokenStream, func: TokenStream) -> TokenStream {
    let func = Function::parse(func);
    let stuff = function_persistenticator(&func);
    stuff
}

fn function_persistenticator(func: &Function) -> TokenStream {
    let vis = &func.vis;
    let fn_token = &func.fn_token;
    let ident = &func.ident;
    let inputs = &func.inputs;
    let output = &func.output;
    let block = &func.block;
    let attrs = &func.attrs;
    // TODO: Deal with empty case
    // Also, make this less horrible...
    let tts = &attrs[0].tts[0];
    let attr = &quote!(#tts).to_string();
    let brackets: &[_] = &['(', ')'];
    let quotes: &[_] = &['"', '"'];
    let attr = attr.trim_matches(brackets);
    let attrs: Vec<&str> = attr.split(',').map(|x| x.trim()).collect();
    let storage: Ident = attrs[0].into();
    let path: &str = attrs[1].trim_matches(quotes);

    let pers_func = quote!{
        extern crate bincode as pers_pc_bincode;
        use std::hash::{Hash, Hasher};
        #vis #fn_token #ident(#inputs) #output
        {
            lazy_static!{
                static ref S: ::std::sync::Mutex<#storage> = ::std::sync::Mutex::new(#storage::new(#path).unwrap());
            };
            let mut s = ::std::collections::hash_map::DefaultHasher::new();


            macro_rules! expand_inputs {
                ($s:ident;) => {};
                ($s:ident; $var:ident : $type:ty) => {
                    $var.hash(&mut $s);
                };
                ($s:ident; $var:ident : $type:ty, $($x:ident : $y:ty,)*) => {
                    expand_inputs!($s; $var : $type);
                    expand_inputs!($s; $($x : $y),*);
                };
            }

            expand_inputs!(s; #inputs,);

            let var_name = format!("{}_{}_{}_{:?}", PREFIX, "fu", stringify!(#ident), s.finish());
            let result: Vec<u8> = S.lock().unwrap().get(&var_name).unwrap();
            match result.len() {
                0 => {
                    // Computing and storing the value
                    let res = #block;
                    S.lock().unwrap().set(&var_name, &pers_pc_bincode::serialize(&res).unwrap()).unwrap();
                    return res;
                },
                _ => {
                    // Fetching the value
                    return pers_pc_bincode::deserialize(&result).unwrap()
                },
            };
        }
    };
    // bypass #46489 (Proc macro hygiene regression)
    let pers_func = pers_func.to_string().parse().unwrap();
    pers_func
    // pers_func.into()
}
