//! For single CPU use, we just make the per-CPU data a global variable.

use quote::quote;
use syn::{Ident, Type};

pub fn gen_offset(symbol: &Ident) -> proc_macro2::TokenStream {
    quote! {
        unsafe { ::core::ptr::addr_of!(#symbol) as usize }
    }
}

pub fn gen_current_ptr(symbol: &Ident, _ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        unsafe { ::core::ptr::addr_of!(#symbol) }
    }
}

pub fn gen_read_current_raw(_symbol: &Ident, _ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        *self.current_ptr()
    }
}

pub fn gen_write_current_raw(_symbol: &Ident, val: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        *(self.current_ptr() as *mut #ty) = #val
    }
}
