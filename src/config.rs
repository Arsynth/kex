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

    pub(super) dedup_enabled: bool,
}

impl<A: AddressFormatting + Clone, B: ByteFormatting + Clone, C: CharFormatting + Clone>
    Config<A, B, C>
{
    /// Create a new config.
    ///
    /// `address_format` - address formatter, used for printing address in the start of each row.
    /// 
    /// `byte_format` - byte formatter, used for printing raw data .
    /// 
    /// `text_format` - ascii formatter, used for printing ascii characters in the end of each row.
    /// 
    /// `dedup_enabled` - row deduplication mode. If `true`, single or multiple duplicated rows will be replaced by single wildcard. 
    pub fn new(
        address_format: Option<A>,
        byte_format: B,
        text_format: Option<C>,
        dedup_enabled: bool,
    ) -> Self {
        Self {
            addr: address_format,
            byte: byte_format,
            text: text_format,
            dedup_enabled,
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
            dedup_enabled: true,
        }
    }
}
