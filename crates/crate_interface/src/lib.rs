#![cfg_attr(not(test), no_std)]

#[macro_export]
macro_rules! define_interface {
    (
        $(#[$attr:ident $($args:tt)*])*
        $vis:vis trait $if_name:ident {
            $($fn:tt)*
        }
    ) => {
        $(#[$attr $($args)*])*
        $vis trait $if_name: Send + Sync {
            $($fn)*
        }

        mod __crate_interface_private {
            use super::$if_name;
            struct __DummyInterface;
            impl $if_name for __DummyInterface {
                __impl_dummy_interface!($if_name, $($fn)*);
            }

            pub(super) static mut __IF_INSTANCE: &dyn $if_name = &__DummyInterface;

            pub(super) fn __get_instance() -> &'static dyn $if_name {
                unsafe{ __IF_INSTANCE }
            }
        }

        $vis fn set_interface(i: &'static dyn $if_name) {
            unsafe { __crate_interface_private::__IF_INSTANCE = i };
        }

    };
}

#[macro_export]
macro_rules! call_interface {
    ($fn:ident $(, $($arg:tt)+)? ) => {
        __crate_interface_private::__get_instance().$fn( $($($arg)+)? )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_dummy_interface {
    (
        $if_name:ident,
        $(#[$attr:ident $($args:tt)*])*
        fn $fn:ident ( &self $(, $($arg:tt)+)? ) $( -> $ret:ty )?;
        $($tail:tt)*
    ) => {
        $(#[$attr $($args)*])*
        #[allow(unused_variables)]
        fn $fn ( &self $(, $($arg)+)? ) $( -> $ret )? {
            unimplemented!("{}::{}()", stringify!($if_name), stringify!($fn));
        }
        __impl_dummy_interface!($if_name, $($tail)*);
    };
    (
        $if_name:ident,
        $(#[$attr:ident $($args:tt)*])*
        fn $fn:ident ( &self $(, $($arg:tt)+)? ) $( -> $ret:ty )? $body:block
        $($tail:tt)*
    ) => {
        $(#[$attr $($args)*])*
        #[allow(unused_variables)]
        fn $fn ( &self $(, $($arg)+)? ) $( -> $ret )? $body
        __impl_dummy_interface!($if_name, $($tail)*);
    };
    ( $if_name:ident, ) => {};
}

#[cfg(test)]
mod test {
    define_interface! {
        trait SimpleIf {
            fn foo(&self) -> u32 {
                123
            }
            fn bar(&self, a: &[u8]);
        }
    }

    struct SimpleIfImpl;

    impl SimpleIf for SimpleIfImpl {
        fn foo(&self) -> u32 {
            456
        }
        fn bar(&self, a: &[u8]) {
            assert_eq!(a[1], 0x24);
        }
    }

    #[test]
    fn test_define_interface() {
        set_interface(&SimpleIfImpl);
        assert_eq!(call_interface!(foo), 456);
        call_interface!(bar, &[0x23, 0x24, 0x25]);
    }
}
