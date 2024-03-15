#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

impl FileId {
    pub const MAX_FILE_ID: u32 = 0x7fff_ffff;

    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        assert!(raw <= Self::MAX_FILE_ID);
        Self(raw)
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id() {
        let file_id = FileId::from_raw(22);
        assert_eq!(file_id.index(), 22);
    }

    #[test]
    #[should_panic]
    fn test_file_id_overflow() {
        FileId::from_raw(FileId::MAX_FILE_ID + 1);
    }

    #[test]
    fn test_file_id_eq() {
        let file_id1 = FileId::from_raw(22);
        let file_id2 = FileId::from_raw(22);
        assert_eq!(file_id1, file_id2);
    }

    #[test]
    fn test_file_id_ne() {
        let file_id1 = FileId::from_raw(22);
        let file_id2 = FileId::from_raw(23);
        assert_ne!(file_id1, file_id2);
    }

    #[test]
    fn test_file_id_ord() {
        let file_id1 = FileId::from_raw(22);
        let file_id2 = FileId::from_raw(23);
        assert!(file_id1 < file_id2);
    }
}
