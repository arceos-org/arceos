use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Error, Fields, Index};

#[proc_macro_derive(TupleForEach)]
pub fn tuple_for_each(item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as DeriveInput);
    if let Data::Struct(strct) = &ast.data {
        if let Fields::Unnamed(_) = strct.fields {
            return impl_for_each(&ast, strct).into();
        }
    }
    Error::new_spanned(
        ast,
        "attribute `tuple_for_each` can only be attached to tuple structs",
    )
    .to_compile_error()
    .into()
}

fn impl_for_each(ast: &DeriveInput, strct: &DataStruct) -> proc_macro2::TokenStream {
    let tuple_name = &ast.ident;
    let macro_name = pascal_to_snake(tuple_name.to_string());
    let macro_for_each = format_ident!("{}_for_each", macro_name);
    let macro_enumerate = format_ident!("{}_enumerate", macro_name);

    let field_num = strct.fields.len();
    let mut for_each = vec![];
    let mut for_each_mut = vec![];
    let mut enumerate = vec![];
    let mut enumerate_mut = vec![];
    for i in 0..field_num {
        let idx = Index::from(i);
        for_each.push(quote!( { let $item = &$tuple.#idx; $code } ));
        for_each_mut.push(quote!( { let $item = &mut $tuple.#idx; $code } ));
        enumerate.push(quote!({
            let $idx = #idx;
            let $item = &$tuple.#idx;
            $code
        }));
        enumerate_mut.push(quote!({
            let $idx = #idx;
            let $item = &mut $tuple.#idx;
            $code
        }));
    }

    quote! {
        impl #tuple_name {
            /// Number of items in the tuple.
            pub const fn len(&self) -> usize {
                #field_num
            }

            /// Whether the tuple has no field.
            pub const fn is_empty(&self) -> bool {
                self.len() == 0
            }
        }

        #[macro_export]
        macro_rules! #macro_for_each {
            ($item:ident in $tuple:ident $code:block) => {
                #(#for_each)*
            };
            ($item:ident in mut $tuple:ident $code:block) => {
                #(#for_each_mut)*
            };
        }

        #[macro_export]
        macro_rules! #macro_enumerate {
            (($idx:ident, $item:ident) in $tuple:ident $code:block) => {
                #(#enumerate)*
            };
            (($idx:ident, $item:ident) in mut $tuple:ident $code:block) => {
                #(#enumerate_mut)*
            };
        }
    }
}

fn pascal_to_snake(pascal: String) -> String {
    let mut ret = String::new();
    for ch in pascal.chars() {
        if ch.is_ascii_uppercase() && !ret.is_empty() {
            ret.push('_')
        }
        ret.push(ch.to_ascii_lowercase());
    }
    ret
}
