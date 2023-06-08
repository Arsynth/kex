
use super::format::*;

pub const DEFAULT_BYTES_PER_ROW: usize = 16;
pub const DEFAULT_NUMBER_OF_GROUPS: usize = 1;

/// Configuration of formatting
pub struct Config<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    pub(super) fmt: Formatters<A, B, T>,

    pub(super) bytes_per_row: usize,
    pub(super) byte_grouping: usize,

    pub(super) decorations: Decorations,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> Config<A, B, T> {
    /// Create new config.
    /// `bytes_per_row` should be greater than zero, otherwise it defaults to [`DEFAULT_BYTES_PER_ROW`]
    pub fn new(
        fmt: Formatters<A, B, T>,
        bytes_per_row: usize,
        byte_grouping: usize,
        third_column_sep: (String, String),
    ) -> Self {
        let bpr = if bytes_per_row == 0 {
            DEFAULT_BYTES_PER_ROW
        } else {
            bytes_per_row
        };

        Self {
            fmt,
            bytes_per_row: bpr,
            byte_grouping,
            decorations: Decorations { third_column_sep },
        }
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    Default for Config<A, B, T>
{
    fn default() -> Config<A, B, T> {
        let fmt: Formatters<A, B, T> = Formatters::new(A::default(), B::default(), T::default());
        Self {
            fmt,
            bytes_per_row: 16,
            byte_grouping: 4,
            decorations: Default::default(),
        }
    }
}

/// Configuration of strict formatting
pub struct StrictConfig<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> {
    pub(super) fmt: Formatters<A, B, T>,
    pub(super) group_size: usize,
    pub(super) number_of_groups: usize,
    pub(super) decorations: Decorations,
}

impl<A: AddressFormatting, B: ByteFormatting, T: ByteFormatting> StrictConfig<A, B, T> {
    /// Create new config.
    /// `number_of_groups` should be greater than zero, otherwise it defaults to [`DEFAULT_NUMBER_OF_GROUPS`]
    pub fn new(
        fmt: Formatters<A, B, T>,
        group_size: usize,
        number_of_groups: usize,
        decorations: Decorations,
    ) -> Self {
        let number_of_groups = match group_size {
            0 => 1,
            _ => {
                if number_of_groups > 0 {
                    number_of_groups
                } else {
                    DEFAULT_NUMBER_OF_GROUPS
                }
            }
        };
        let group_size = if group_size > 0 {
            group_size
        } else {
            DEFAULT_BYTES_PER_ROW
        };

        Self {
            fmt,
            group_size,
            number_of_groups,
            decorations,
        }
    }
}

impl<A: AddressFormatting + Default, B: ByteFormatting + Default, T: ByteFormatting + Default>
    Default for StrictConfig<A, B, T>
{
    fn default() -> Self {
        let fmt: Formatters<A, B, T> = Formatters::new(A::default(), B::default(), T::default());
        Self {
            fmt,
            group_size: 0,
            number_of_groups: 0,
            decorations: Default::default(),
        }
    }
}
