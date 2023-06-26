pub const DEFAULT_BYTES_PER_ROW: usize = 16;
pub const DEFAULT_GROUP_SIZE: usize = 4;
pub const DEFAULT_NUMBER_OF_GROUPS: usize = 4;

/// Requirement for providing byte portions
pub enum GroupAtomicity {
    /// Bytes will be provided in portions strictly according to grouping.
    /// In that case, buffering will be used
    Required,
    /// Bytes will be provided as soon as new data available in [`crate::Printer`]
    Optional,
}

/// Byte formatting style
#[derive(Clone)]
pub enum Groupping {
    /// Single group with bytes count
    RowWide(usize),
    /// Group with repeat count. `bytes per row` = `group size` * `number of groups`
    RepeatingGroup(Group, usize),
}

impl Groupping {
    pub(crate) fn separator(&self) -> Vec<u8> {
        match self {
            Groupping::RowWide(_) => vec![],
            Groupping::RepeatingGroup(g, _) => g.separator.clone(),
        }
    }

    pub(crate) fn is_aligned_at(&self, number: usize) -> bool {
        let bpr = self.bytes_per_row();
        assert!(
            bpr > number,
            "is_aligned_at(): Trying to exceed maximum row length"
        );

        match self {
            Groupping::RowWide(_) => number == 0,
            Groupping::RepeatingGroup(g, _) => number % g.size == 0,
        }
    }

    pub(crate) fn bytes_per_row(&self) -> usize {
        match self {
            Groupping::RowWide(r) => *r,
            Groupping::RepeatingGroup(g, rep) => g.size * rep,
        }
    }

    pub(crate) fn group_of_byte(&self, number: usize) -> usize {
        assert!(
            self.bytes_per_row() >= number,
            "group_of_byte():Trying to exceed maximum row length"
        );
        match self {
            Groupping::RowWide(_) => 0,
            Groupping::RepeatingGroup(g, _) => {
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
            Groupping::RepeatingGroup(g, _) => {
                let group_num = self.group_of_byte(number);
                g.size - (number - g.size * group_num)
            }
        }
    }

    pub(crate) fn max_group_size(&self) -> usize {
        match self {
            Groupping::RowWide(r) => *r,
            Groupping::RepeatingGroup(g, _) => g.size,
        }
    }
}

impl Default for Groupping {
    fn default() -> Self {
        Self::RepeatingGroup(Default::default(), DEFAULT_NUMBER_OF_GROUPS)
    }
}

/// Group of bytes, separated by arbitary printable characters
#[derive(Clone)]
pub struct Group {
    /// Number of bytes in the group
    pub(super) size: usize,
    pub(super) separator: Vec<u8>,
}

impl Group {
    /// Create group.
    /// 
    /// `size` - group width in bytes.
    /// 
    /// `separator` - separating chars between groups.
    pub fn new(size: usize, separator: &str) -> Self {
        Self {
            size,
            separator: Vec::from(separator.as_bytes()),
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new(DEFAULT_GROUP_SIZE, " ")
    }
}
