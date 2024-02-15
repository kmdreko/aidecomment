//! This crate provides a macro that can be used to extract a summary and
//! description for an OpenAPI operation from doc comments. This crate supports
//! [axum](https://crates.io/crates/axum) and integrates this information with
//! [aide](https://crates.io/crates/aide).

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Expr, FnArg, Ident, ItemFn, Lit};

/// An attribute to provide the summary and description from a doc comment.
///
/// This will take the info from the doc comment and inject it into an axum
/// handler function as an aide `OperationInput` implementation. The first
/// paragraph is interpretted as the summary and everything else is interpretted
/// as the description.
///
/// ```
/// # use aidecomment::aidecomment;
/// /// This is a summary
/// ///
/// /// This is a longer description of the endpoint that is expected to be much
/// /// more detailed and may span more lines than the first paragraph summary.
/// #[aidecomment]
/// async fn my_handler() -> &'static str {
///     "hello world"
/// }
/// # fn can_compile_as_axum_handler() {
/// #     use axum::{Router, routing::get};
/// #     Router::<()>::new().route("/", get(my_handler));
/// # }
/// # fn can_compile_as_aide_operation_handler() {
/// #     use aide::axum::{ApiRouter, routing::get};
/// #     ApiRouter::<()>::new().api_route("/", get(my_handler));
/// # }
/// ```
///
/// The external dependencies `axum` and `aide` need to be available. Tested
/// with versions: `axum@0.7.4`, `aide@0.13.2`.
#[proc_macro_attribute]
pub fn aidecomment(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fn_def = syn::parse_macro_input!(item as ItemFn);

    let doc_comments = fn_def
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(nvmeta) => Some(nvmeta),
            _ => None,
        })
        .filter(|nvmeta| nvmeta.path.get_ident().map(|i| i.to_string()) == Some("doc".to_owned()))
        .filter_map(|nvmeta| match &nvmeta.value {
            Expr::Lit(literal) => Some(literal),
            _ => None,
        })
        .filter_map(|literal| match &literal.lit {
            Lit::Str(string) => Some(string.value()),
            _ => None,
        })
        .collect::<Vec<_>>();

    let doc_comment = doc_comments.join("\n");
    let mut lines = doc_comment.lines().collect::<Vec<_>>();

    // separate summary from description
    let first_empty_idx = lines
        .iter()
        .position(|line| line.trim().is_empty())
        .unwrap_or(lines.len());

    let summary = lines.drain(0..first_empty_idx).collect::<Vec<_>>().join("");
    let summary = summary.trim();

    let description = lines.join("\n");
    let description = description.trim();

    let struct_name = fn_def.sig.ident.to_string() + "_AideComment";
    let struct_name = Ident::new(&struct_name, Span::mixed_site());

    let vis = fn_def.vis.clone();

    let arg = syn::parse_str::<FnArg>(&format!("_: {struct_name}")).unwrap();
    fn_def.sig.inputs.insert(0, arg);

    quote! {
        #vis struct #struct_name;

        impl ::aide::OperationInput for #struct_name {
            fn operation_input(_ctx: &mut ::aide::gen::GenContext, operation: &mut ::aide::openapi::Operation) {
                operation.summary = Some(#summary.to_owned());
                operation.description = Some(#description.to_owned());
            }
        }

        #[::axum::async_trait]
        impl<S> ::axum::extract::FromRequestParts<S> for #struct_name {
            type Rejection = ::std::convert::Infallible;
            async fn from_request_parts(
                _parts: &mut ::axum::http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                Ok(#struct_name)
            }
        }

        #fn_def
    }.into()
}
