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

                skool_cookie::future::ready(Self::from_req(req))
            }
        }

        impl skool_cookie::CookieDough for #name {
            const COOKIE_NAME: &'static str = #cookie_name;

            fn from_req(req: &impl skool_cookie::UsableRequest) -> Result<Self, skool_cookie::CookieError> {
                match skool_cookie::UsableRequest::cookie(req, Self::COOKIE_NAME) {
                    Some(cookie) => {
                        let conf = skool_cookie::cookie_config(req);

                        skool_cookie::eat_paranoid_cookie(cookie, &conf.key).map_err(|e| e.into())
                    },
                    None => Err(skool_cookie::CookieError::MissingCookie),
                }
            }
        }
    }.into()
}
