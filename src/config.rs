
use super::format::*;

/// Configuration of formatting
pub struct Config<A: AddressFormatting, B: ByteFormatting, T: CharFormatting> {
    pub(super) fmt: Formatters<A, B, T>,
    pub(super) decorations: Decorations,
}

impl<A: AddressFormatting, B: ByteFormatting, T: CharFormatting> Config<A, B, T> {
    /// Create new config.
    /// `bytes_per_row` should be greater than zero, otherwise it defaults to [`DEFAULT_BYTES_PER_ROW`]
    pub fn new(
        fmt: Formatters<A, B, T>,
        decorations: Decorations,
    ) -> Self {
        Self {
            fmt,
            decorations,
        }
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: CharFormatting + Default>
    Default for Config<A, B, T>
{
    fn default() -> Config<A, B, T> {
        let fmt: Formatters<A, B, T> = Formatters::new(A::default(), B::default(), T::default());
        Self {
            fmt,
            decorations: Default::default(),
        }
    }
}

pub struct Decorations {
    pub(super) third_column_sep: (Vec<u8>, Vec<u8>),
}

impl Decorations {
    pub fn new(third_column_sep: (String, String)) -> Self {
        Self {
            third_column_sep: (
                Vec::from(third_column_sep.0.as_bytes()),
                Vec::from(third_column_sep.1.as_bytes()),
            )
        }
    }
}

impl Default for Decorations {
    fn default() -> Self {
        Self {
            third_column_sep: (Vec::from("|".as_bytes()), Vec::from("|".as_bytes())),
        }
    }
}