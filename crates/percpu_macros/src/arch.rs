use quote::{format_ident, quote};
use syn::{Ident, Type};

fn macos_unimplemented(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        #[cfg(not(target_os = "macos"))]
        { #item }
        #[cfg(target_os = "macos")]
        unimplemented!()
    }
}

pub fn gen_offset(symbol: &Ident) -> proc_macro2::TokenStream {
    quote! {
        let value: usize;
        unsafe {
            #[cfg(target_arch = "x86_64")]
            ::core::arch::asm!(
                "movabs {0}, offset {VAR}",
                out(reg) value,
                VAR = sym #symbol,
            );
            #[cfg(target_arch = "aarch64")]
            ::core::arch::asm!(
                "movz {0}, #:abs_g0_nc:{VAR}",
                out(reg) value,
                VAR = sym #symbol,
            );
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            ::core::arch::asm!(
                "lui {0}, %hi({VAR})",
                "addi {0}, {0}, %lo({VAR})",
                out(reg) value,
                VAR = sym #symbol,
            );
        }
        value
    }
}

pub fn gen_current_ptr(symbol: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    macos_unimplemented(quote! {
        let base: usize;
        #[cfg(target_arch = "x86_64")]
        {
            // `__PERCPU_SELF_PTR` stores GS_BASE, which is defined in crate `percpu`.
            ::core::arch::asm!(
                "mov {0}, gs:[offset __PERCPU_SELF_PTR]",
                "add {0}, offset {VAR}",
                out(reg) base,
                VAR = sym #symbol,
            );
            base as *const #ty
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            #[cfg(target_arch = "aarch64")]
            ::core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) base);
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            ::core::arch::asm!("mv {}, gp", out(reg) base);
            (base + self.offset()) as *const #ty
        }
    })
}

pub fn gen_read_current_raw(symbol: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    let ty_str = quote!(#ty).to_string();
    let rv64_op = match ty_str.as_str() {
        "bool" => "lbu",
        "u8" => "lbu",
        "u16" => "lhu",
        "u32" => "lwu",
        "u64" => "ld",
        "usize" => "ld",
        _ => unreachable!(),
    };
    let rv64_asm = quote! {
        ::core::arch::asm!(
            "lui {0}, %hi({VAR})",
            "add {0}, {0}, gp",
            concat!(#rv64_op, " {0}, %lo({VAR})({0})"),
            out(reg) value,
            VAR = sym #symbol,
        )
    };

    let (x64_asm, x64_reg) = if ["bool", "u8"].contains(&ty_str.as_str()) {
        (
            "mov {0}, byte ptr gs:[offset {VAR}]".into(),
            format_ident!("reg_byte"),
        )
    } else {
        let (x64_mod, x64_ptr) = match ty_str.as_str() {
            "u16" => ("x", "word"),
            "u32" => ("e", "dword"),
            "u64" => ("r", "qword"),
            "usize" => ("r", "qword"),
            _ => unreachable!(),
        };
        (
            format!("mov {{0:{x64_mod}}}, {x64_ptr} ptr gs:[offset {{VAR}}]"),
            format_ident!("reg"),
        )
    };
    let x64_asm = quote! {
        ::core::arch::asm!(#x64_asm, out(#x64_reg) value, VAR = sym #symbol)
    };

    let gen_code = |asm_stmt| {
        if ty_str.as_str() == "bool" {
            quote! {
                let value: u8;
                #asm_stmt;
                value != 0
            }
        } else {
            quote! {
                let value: #ty;
                #asm_stmt;
                value
            }
        }
    };

    let rv64_code = gen_code(rv64_asm);
    let x64_code = gen_code(x64_asm);
    macos_unimplemented(quote! {
        #[cfg(target_arch = "riscv64")]
        { #rv64_code }
        #[cfg(target_arch = "x86_64")]
        { #x64_code }
        #[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
        { *self.current_ptr() }
    })
}

pub fn gen_write_current_raw(symbol: &Ident, val: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    let ty_str = quote!(#ty).to_string();
    let ty_fixup = if ty_str.as_str() == "bool" {
        format_ident!("u8")
    } else {
        format_ident!("{}", ty_str)
    };

    let rv64_op = match ty_str.as_str() {
        "bool" => "sb",
        "u8" => "sb",
        "u16" => "sh",
        "u32" => "sw",
        "u64" => "sd",
        "usize" => "sd",
        _ => unreachable!(),
    };
    let rv64_code = quote! {
        ::core::arch::asm!(
            "lui {0}, %hi({VAR})",
            "add {0}, {0}, gp",
            concat!(#rv64_op, " {1}, %lo({VAR})({0})"),
            out(reg) _,
            in(reg) #val as #ty_fixup,
            VAR = sym #symbol,
        );
    };

    let (x64_asm, x64_reg) = if ["bool", "u8"].contains(&ty_str.as_str()) {
        (
            "mov byte ptr gs:[offset {VAR}], {0}".into(),
            format_ident!("reg_byte"),
        )
    } else {
        let (x64_mod, x64_ptr) = match ty_str.as_str() {
            "u16" => ("x", "word"),
            "u32" => ("e", "dword"),
            "u64" => ("r", "qword"),
            "usize" => ("r", "qword"),
            _ => unreachable!(),
        };
        (
            format!("mov {x64_ptr} ptr gs:[offset {{VAR}}], {{0:{x64_mod}}}"),
            format_ident!("reg"),
        )
    };
    let x64_code = quote! {
        ::core::arch::asm!(#x64_asm, in(#x64_reg) #val as #ty_fixup, VAR = sym #symbol)
    };

    macos_unimplemented(quote! {
        #[cfg(target_arch = "riscv64")]
        { #rv64_code }
        #[cfg(target_arch = "x86_64")]
        { #x64_code }
        #[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
        { *(self.current_ptr() as *mut #ty) = #val }
    })
}
