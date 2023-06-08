/// Assertion enabled only when the `debug` feature is enabled.
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! jbd_assert {
    ($e:expr) => {
        assert!(
            $e,
            "jbd-rs: jbd_assert failed at {}:{}:{}: {}",
            file!(),
            line!(),
            column!(),
            stringify!($e),
        );
    };
}

/// Assertion enabled only when the `debug` feature is enabled.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! jbd_assert {
    ($e:expr) => {};
}
