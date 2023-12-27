#[macro_use]
pub mod macros {
	macro_rules! r_assert {
    ($cond:expr, $($args:tt)+) => {{
      if !$cond {
        return Err(format!($($args)+))
      }
    }};
  }

	macro_rules! r_panic {
    ($($args:tt)+) => {{
      return Err(format!($($args)+))
    }};
  }

	pub(crate) use r_assert;
	pub(crate) use r_panic;
}
