#[cfg(feature = "debug")]
#[macro_export]
macro_rules! jbd_assert {
    ($e:expr) => {
        assert!(
            $e,
            "jbd-rs: assertion failed at {}:{}:{}: {}",
            file!(),
            line!(),
            column!(),
            stringify!($e),
        );
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! jbd_assert {
    ($e:expr) => {};
}
