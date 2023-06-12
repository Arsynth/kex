use super::format::*;

/// Configuration of formatting
#[derive(Clone)]
pub struct Config<
    A: AddressFormatting + Clone,
    B: ByteFormatting + Clone,
    C: CharFormatting + Clone,
> {
    pub(super) addr: Option<A>,
    pub(super) byte: B,
    pub(super) text: Option<C>,
}

impl<A: AddressFormatting + Clone, B: ByteFormatting + Clone, C: CharFormatting + Clone>
    Config<A, B, C>
{
    /// Create new config.
    /// `bytes_per_row` should be greater than zero, otherwise it defaults to [`DEFAULT_BYTES_PER_ROW`]
    pub fn new(address_format: Option<A>, byte_format: B, text_format: Option<C>) -> Self {
        Self {
            addr: address_format,
            byte: byte_format,
            text: text_format,
        }
    }
}

impl<
        A: AddressFormatting + Default + Clone,
        B: ByteFormatting + Default + Clone,
        C: CharFormatting + Default + Clone,
    > Default for Config<A, B, C>
{
    fn default() -> Config<A, B, C> {
        Self {
            addr: Some(A::default()),
            byte: B::default(),
            text: Some(C::default()),
        }
    }
}
