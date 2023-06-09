pub const DEFAULT_BYTES_PER_ROW: usize = 16;
pub const DEFAULT_GROUP_SIZE: usize = 4;
pub const DEFAULT_NUMBER_OF_GROUPS: usize = 4;

pub trait ByteOrdered {
    /// Requirement for byte portions passing in the `format(...)` function
    fn byte_order(&self) -> ByteOrder;

    fn groupping(&self) -> &Groupping;
}

/// Requirement for providing byte portions
pub enum ByteOrder {
    /// Bytes will be provided in portions strictly according to grouping.
    /// In that case, buffering will be used
    Strict,
    /// Bytes will be provided as soon as new data available in [`super::Printer`]
    Relaxed,
}

/// Byte formatting style
pub enum Groupping {
    /// Single group with bytes count
    RowWide(usize),
    /// Group with repeat count. `bytes per row` = `group size` * `number of groups`
    RepeatingGroup(Group, usize),
    /// Number of bytes in row does not depend on group size, it is specified directly.
    ///
    /// ## Note
    /// If the bytes per row is not aligned to group size, last group will be incomplete sized
    BytesPerRow(usize, Group),
}

impl Groupping {
    pub(crate) fn separator(&self) -> String {
        match self {
            Groupping::RowWide(_) => "".to_string(),
            Groupping::RepeatingGroup(g, _) => g.separator.clone(),
            Groupping::BytesPerRow(_, g) => g.separator.clone(),
        }
    }

    pub(crate) fn is_aligned_at(&self, number: usize) -> bool {
        let bpr = self.bytes_per_row();
        assert!(
            bpr >= number,
            "is_aligned_at(): Trying to exceed maximum row length"
        );

        match self {
            Groupping::RowWide(_) => number == 0,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let rem = bpr % g.size;
                if rem == 0 {
                    number % g.size == 0
                } else {
                    let rem_group = rem;
                    let n_aligned_groups = (bpr - rem_group) / g.size;

                    number == n_aligned_groups * g.size
                }
            }
        }
    }

    pub(crate) fn is_aligned_range(&self, number: usize, len: usize) -> bool {
        let bpr = self.bytes_per_row();
        assert!(
            bpr >= number + len,
            "is_aligned_range(): Trying to exceed maximum row length"
        );

        match self {
            Groupping::RowWide(_) => number == 0 && len == bpr,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let rem = bpr % g.size;
                if rem == 0 {
                    number % g.size == 0 && len == g.size
                } else {
                    let rem_group = rem;
                    let n_aligned_groups = (bpr - rem_group) / g.size;

                    number == n_aligned_groups * g.size && len == rem_group
                }
            }
        }
    }

    pub(crate) fn bytes_per_row(&self) -> usize {
        match self {
            Groupping::RowWide(r) => *r,
            Groupping::RepeatingGroup(g, rep) => g.size * rep,
            Groupping::BytesPerRow(r, _) => *r,
        }
    }

    pub(crate) fn number_of_groups(&self) -> usize {
        match self {
            Groupping::RowWide(_) => 1,
            Groupping::RepeatingGroup(_, rep) => *rep,
            Groupping::BytesPerRow(r, g) => {
                let rem = r % g.size;
                if rem == 0 {
                    r / g.size
                } else {
                    (r - rem) / g.size + 1
                }
            }
        }
    }

    pub(crate) fn group_of_byte(&self, number: usize) -> usize {
        assert!(
            self.bytes_per_row() >= number,
            "group_of_byte():Trying to exceed maximum row length"
        );
        match self {
            Groupping::RowWide(_) => 0,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let group_size = g.size;
                let rem = number % group_size;
                (number - rem) / group_size
            }
        }
    }

    pub(crate) fn bytes_left_in_group_after(&self, number: usize) -> usize {
        assert!(
            self.bytes_per_row() >= number,
            "bytes_left_in_group_after(): Trying to exceed maximum row length"
        );
        match self {
            Groupping::RowWide(r) => r - number,
            Groupping::RepeatingGroup(g, _) | Groupping::BytesPerRow(_, g) => {
                let group_num = self.group_of_byte(number);
                g.size - (number - g.size * group_num)
            }
        }
    }
}

impl Default for Groupping {
    fn default() -> Self {
        Self::RepeatingGroup(Default::default(), DEFAULT_NUMBER_OF_GROUPS)
    }
}

pub struct Group {
    /// Number of bytes in the group
    pub(super) size: usize,
    pub(super) separator: String,
}

impl Group {
    pub fn new(size: usize, separator: &str) -> Self {
        Self {
            size,
            separator: separator.to_string(),
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new(DEFAULT_GROUP_SIZE, " ")
    }
}