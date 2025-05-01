use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Token, parse::Parse, parse_macro_input, punctuated::Punctuated};

/// The `#[observable]` attribute macro automatically derives the `Observable` trait for a struct.
///
/// It generates the `Observer` and `Changes` types specific to the struct, and implements `Observable` for it.
///
/// # Syntax:
/// ```rust
/// #[observable(<derives>)]
/// pub struct YourStruct {
///     // fields
/// }
/// ```
///
/// # Parameters:
/// - `<derives>`: Comma-separated list of derive attributes (e.g., `Debug`, `PartialEq`, `Eq`) applied to the generated `Observer` and `Changes` types.
///
/// # Example:
/// ```rust
/// #[observable(Debug, PartialEq, Eq)]
/// pub struct Example {
///     a: i32,
///     b: i16,
/// }
/// ```
///
/// # Generated Code:
/// - `Observer<YourStruct>`: A struct holding the previous values of each field, implementing `Default`.
/// - `Changes<YourStruct>`: A struct representing the changes between the current and previous state.
/// - `pull_changes`: A method that computes and returns the changes, while updating the observer.
#[proc_macro_attribute]
pub fn observable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Attr);
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let observer_name = format_ident!("{}Observer", struct_name);
    let changes_name = format_ident!("{}Changes", struct_name);

    let syn::Data::Struct(data_struct) = &input.data else {
        return syn::Error::new_spanned(struct_name, "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let fields = &data_struct.fields;
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let derives: Vec<_> = attr.derives.iter().collect();

    let expanded = quote! {
        #input

        const _: () = {
            #[derive(Default, #(#derives),*)]
            pub struct #observer_name<'a> {
                #( #field_names: <#field_types as Observable>::Observer<'a>, )*
            }

            #[derive(#(#derives),*)]
            pub struct #changes_name<'a> {
                #( #field_names: <#field_types as Observable>::Changes<'a>, )*
            }

            impl Observable for #struct_name {
                type Observer<'a> = #observer_name<'a>;
                type Changes<'a> = #changes_name<'a>;

                fn pull_changes(&self, observer: &mut Self::Observer<'_>) -> Self::Changes<'_> {
                    #changes_name {
                        #( #field_names: self.#field_names.pull_changes(&mut observer.#field_names), )*
                    }
                }
            }
        };
    };

    TokenStream::from(expanded)
}

struct Attr {
    derives: Punctuated<syn::Ident, Token![,]>,
}
impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Attr {
            derives: Punctuated::parse_terminated(input)?,
        })
    }
}
