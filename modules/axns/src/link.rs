//! Module for defining static resources in the `axns_resource` section under
//! different operating systems.
//!
//! The implementation is based on `link_section` attributes and platform-specific
//! linker support.

#[cfg(any(target_os = "linux", target_os = "none"))]
#[macro_use]
pub mod linux {
    //! Module for defining static resources in the `axns_resource` section under Linux and
    //! other similar ELF environments.

    unsafe extern "C" {
        fn __start_axns_resource();
        fn __stop_axns_resource();
    }

    /// Defines a static resource of the specified type and default value,
    /// placing it in the custom linker section `axns_resource`.
    ///
    /// # Parameters
    /// - `$ty`: The type of the static resource.
    /// - `$default`: The default value to initialize the resource with.
    ///
    /// # Generated Items
    /// - A static variable `RES` of type `$ty`, initialized with `$default`.
    /// - A function `res_ptr()` that returns a raw pointer to the resource as `*const u8`.
    ///
    /// # Example
    /// ```rust,ignore
    /// def_static_resource!(u32, 0);
    /// ```
    #[macro_export]
    macro_rules! def_static_resource {
        (RES, $ty: ty, $default: expr) => {
            #[unsafe(link_section = "axns_resource")]
            static RES: $ty = $default;
        };
    }

    /// Returns a raw pointer to the static resource defined in the `axns_resource` section.
    pub fn section_start() -> *const u8 {
        __start_axns_resource as *const u8
    }

    /// Returns a raw pointer to the end of the `axns_resource` section.
    pub fn section_end() -> *const u8 {
        __stop_axns_resource as *const u8
    }
}

#[cfg(target_os = "macos")]
#[macro_use]
pub mod macho {
    //! Module for defining static resources in the `axns_resource` section under macOS
    //! and other similar mach-O environments.

    unsafe extern "Rust" {
        #[link_name = "\u{1}section$start$__AXNS$__axns_resource"]
        static AXNS_RESOURCE_START: *const u8;
        #[link_name = "\u{1}section$end$__AXNS$__axns_resource"]
        static AXNS_RESOURCE_END: *const u8;
    }

    #[macro_export]
    /// Defines a static resource of the specified type and default value,
    /// placing it in the custom linker section `axns_resource`.
    ///
    /// # Parameters
    /// - `$ty`: The type of the static resource.
    /// - `$default`: The default value to initialize the resource with.
    ///
    /// # Generated Items
    /// - A static variable `RES` of type `$ty`, initialized with `$default`.
    /// - A function `res_ptr()` that returns a raw pointer to the resource as `*const u8`.
    ///
    /// # Example
    /// ```rust,ignore
    /// def_static_resource!(u32, 0);
    /// ```
    macro_rules! def_static_resource {
        (RES, $ty: ty, $default: expr) => {
            #[unsafe(link_section = "__AXNS,axns_resource")]
            static RES: $ty = $default;
        };
    }

    /// Returns a pointer to the start of `axns_resource` section.
    pub fn section_start() -> *const u8 {
        unsafe { AXNS_RESOURCE_START }
    }

    /// Returns a pointer to the end of `axns_resource` section.
    pub fn section_end() -> *const u8 {
        unsafe { AXNS_RESOURCE_END }
    }
}

#[cfg(any(target_os = "linux", target_os = "none"))]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macho::*;
