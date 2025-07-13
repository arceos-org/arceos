#![allow(unused_macros)]

macro_rules! define_api_type {
    ($( $(#[$attr:meta])* $vis:vis type $name:ident $(<$($generic:tt)*>)? ; )+) => {
        $(
            $vis use $crate::imp::$name;
        )+
    };
    ( @cfg $feature:literal; $( $(#[$attr:meta])* $vis:vis type $name:ident $(<$($generic:tt)*>)? ; )+ ) => {
        $(
            #[cfg(feature = $feature)]
            $(#[$attr])*
            $vis use $crate::imp::$name;
            #[cfg(all(feature = "dummy-if-not-enabled", not(feature = $feature)))]
            $(#[$attr])*
            $vis struct $name $(<$($generic)*>)? $(where $($generic)*: {})?;
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
    ($( $(#[$attr:meta])* $vis:vis unsafe fn $name:ident( $($arg:ident : $type:ty),* $(,)? ) $( -> $ret:ty )? ; )+) => {
        $(
            $(#[$attr])*
            $vis unsafe fn $name( $($arg : $type),* ) $( -> $ret )? {
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
    (
        @cfg $feature:literal;
        $( $(#[$attr:meta])* $vis:vis unsafe fn $name:ident( $($arg:ident : $type:ty),* $(,)? ) $( -> $ret:ty )? ; )+
    ) => {
        $(
            #[cfg(feature = $feature)]
            $(#[$attr])*
            $vis unsafe fn $name( $($arg : $type),* ) $( -> $ret )? {
                $crate::imp::$name( $($arg),* )
            }

            #[allow(unused_variables)]
            #[cfg(all(feature = "dummy-if-not-enabled", not(feature = $feature)))]
            $(#[$attr])*
            $vis unsafe fn $name( $($arg : $type),* ) $( -> $ret )? {
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

macro_rules! cfg_dma {
    ($($item:item)*) => { _cfg_common!{ "dma" $($item)* } }
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

macro_rules! cfg_async_preempt {
    ($($item:item)*) => { _cfg_common!{ "async-preempt" $($item)* } }
}

macro_rules! cfg_async_thread {
    ($($item:item)*) => { _cfg_common!{ "async-thread" $($item)* } }
}

macro_rules! cfg_async_single {
    ($($item:item)*) => { _cfg_common!{ "async-single" $($item)* } }
}

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "async-thread", feature = "async-single"))]
            $item
        )*
    }
}
