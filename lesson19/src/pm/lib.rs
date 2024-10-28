use proc_macro::{Ident, TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Парсим входной токен как синтаксическое дерево
    let input = parse_macro_input!(input as DeriveInput);
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

    f.push_str(")");

    f.parse().unwrap()
}
