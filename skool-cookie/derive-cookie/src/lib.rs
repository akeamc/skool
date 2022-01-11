extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Cookie, attributes(cookie_name))]
pub fn derive_cookie(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let input = parse_macro_input!(input as DeriveInput);

    impl_cookie(&input)
}

fn impl_cookie(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attr = ast
        .attrs
        .iter()
        .find(|a| a.path.is_ident("cookie_name"))
        .expect("cookie_name not set");
    let cookie_name = attr.parse_args::<syn::LitStr>().unwrap().value();

    quote! {
        impl skool_cookie::FromRequest for #name {
            type Error = skool_cookie::CookieError;

            type Future = skool_cookie::future::Ready<Result<Self, Self::Error>>;

            fn from_request(req: &skool_cookie::HttpRequest, _: &mut skool_cookie::Payload) -> Self::Future {
                use skool_cookie::CookieDough;

                match req.cookie(Self::COOKIE_NAME) {
                    Some(cookie) => {
                        let conf = skool_cookie::cookie_config(&req);

                        match skool_cookie::eat_paranoid_cookie(cookie, &conf.key) {
                        Ok(v) => skool_cookie::future::ok(v),
                        Err(e) => skool_cookie::future::err(e.into()),
                    }},
                    None => skool_cookie::future::err(Self::Error::MissingCookie),
                }
            }
        }

        impl skool_cookie::CookieDough for #name {
            const COOKIE_NAME: &'static str = #cookie_name;
        }
    }.into()
}
