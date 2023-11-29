// https://github.com/wcampbell0x2a/assert_hex/blob/master/src/lib.rs
/// Asserts that two expressions are equal to each other
///
/// On panic, this macro will print values of the expressions in their
/// `{:#x?}` (hexadecimal) representation
#[macro_export]
macro_rules! assert_eq_hex {
    ($left:expr, $right:expr $(,)?) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion `left == right` failed
  left: {:X?}
 right: {:X?}"#, &*left_val, &*right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion `left == right` failed: {}
  left: {:X?}
 right: {:X?}"#, format_args!($($arg)+), &*left_val, &*right_val)
                }
            }
        }
    });
}
