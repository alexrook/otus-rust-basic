use std::fmt::Debug;

use proc_macro::{TokenStream, TokenTree};
use quote::quote;

#[proc_macro_derive(MyDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Парсим входной токен как синтаксическое дерево
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    // Генерируем реализацию трейта Debug
    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, stringify!(#name))
            }
        }
    };

    // Преобразуем сгенерированный код в токен-поток
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn add_one(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    if let Some(TokenTree::Literal(lit)) = iter.next() {
        // Преобразуем литерал в целое число
        let value: i64 = lit.to_string().parse().unwrap();
        // Возвращаем результат как новый `TokenStream`
        let new_value = value + 1;
        format!("{}", new_value).parse().unwrap()
    } else {
        panic!("Expected a literal");
    }
}

//Процедурный макрос, принимающий набор строковых литералов - имён функций.
// Макрос должен возвращать кортеж из возвращаемых значений тех функций,
//  в именах которых чётное количество символов.
// Число функций может быть произвольным.
//    Пример:
//  let (fo_result, fooo_result) = my_macro!(""fo"", ""foo"", ""fooo"");
#[proc_macro]
pub fn even_func_name(input: TokenStream) -> TokenStream {
    let mut f = input
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Ident(ident) if ident.to_string().len() % 2_usize == 0 => Some(ident),
            _ => None,
        })
        .fold("(".to_string(), |mut t, ident| {
            t.push_str(&ident.to_string());
            t.push_str("(),");
            t
        });

    f.push(')');

    f.parse().unwrap()
}

struct Input {
    functions: syn::punctuated::Punctuated<FuncName, syn:: Token![,]>,
}

impl Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str: String = self
            .functions
            .iter()
            .fold("Input{".to_string(), |mut acc, i| {
                acc.push_str(&i.name.to_string());
                acc.push(',');
                acc
            });
        str.push('}');
        f.write_str(&str)
    }
}

#[derive(Debug)]
struct FuncName {
    name: syn::Ident,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            functions: syn::punctuated::Punctuated::parse_terminated(input)?,
        })
    }
}

impl syn::parse::Parse for FuncName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FuncName {
            name: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn even_func_name_v2(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    let outs = input.functions.iter().filter_map(|func| {
        let func_name = func.name.to_string();
        if func_name.len() % 2 == 0 {
            Some(&func.name)
        } else {
            None
        }
    });

    quote! {
        (
            #(
                #outs(),
            )*
        )
    }
    .into()
}
