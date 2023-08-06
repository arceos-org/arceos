#![allow(unused_macros)]

macro_rules! define_api_type {
    ($( $(#[$attr:meta])* $vis:vis type $name:ident; )+) => {
        $(
            $vis use $crate::imp::$name;
        )+
    };
    ( @cfg $feature:literal; $( $(#[$attr:meta])* $vis:vis type $name:ident; )+ ) => {
        $(
            #[cfg(feature = $feature)]
            $(#[$attr])*
            $vis use $crate::imp::$name;

            #[cfg(all(feature = "dummy-if-not-enabled", not(feature = $feature)))]
            $(#[$attr])*
            $vis struct $name;
        )+
    };
}

macro_rules! define_api {
    ($( $(#[$attr:meta])* $vis:vis fn $name:ident( $($arg:ident : $type:ty),* $(,)? ) $( -> $ret:ty )? ; )+) => {
        $(
            $(#[$attr])*
            $vis fn $name( $($arg : $type),* ) $( -> $ret )? {
                $crate::imp::$name( $($arg),* )
            }
        )+
    };
    (
        @cfg $feature:literal;
        $( $(#[$attr:meta])* $vis:vis fn $name:ident( $($arg:ident : $type:ty),* $(,)? ) $( -> $ret:ty )? ; )+
    ) => {
        $(
            #[cfg(feature = $feature)]
            $(#[$attr])*
            $vis fn $name( $($arg : $type),* ) $( -> $ret )? {
                $crate::imp::$name( $($arg),* )
            }

            #[allow(unused_variables)]
            #[cfg(all(feature = "dummy-if-not-enabled", not(feature = $feature)))]
            $(#[$attr])*
            $vis fn $name( $($arg : $type),* ) $( -> $ret )? {
                unimplemented!(stringify!($name))
            }
        )+
    };
}

macro_rules! _cfg_common {
    ( $feature:literal $($item:item)*  ) => {
        $(
            #[cfg(feature = $feature)]
            $item
        )*
    }
}

macro_rules! cfg_alloc {
    ($($item:item)*) => { _cfg_common!{ "alloc" $($item)* } }
}

macro_rules! cfg_fs {
    ($($item:item)*) => { _cfg_common!{ "fs" $($item)* } }
}

macro_rules! cfg_net {
    ($($item:item)*) => { _cfg_common!{ "net" $($item)* } }
}

macro_rules! cfg_display {
    ($($item:item)*) => { _cfg_common!{ "display" $($item)* } }
}

macro_rules! cfg_task {
    ($($item:item)*) => { _cfg_common!{ "multitask" $($item)* } }
}
