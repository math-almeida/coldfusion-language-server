//! Maps paths to compact integer ids. We don't care about clearings paths which
//! no longer exist -- the assumption is total size of paths we ever look at is
//! not too big.
use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;
use indexmap::IndexSet;

use crate::{FileId, VirtualFsPath};

/// Structure to map between [`VirtualFsPath`] and [`FileId`].
#[derive(Default)]
pub(crate) struct PathInterner {
    map: IndexSet<VirtualFsPath, BuildHasherDefault<FxHasher>>,
}

impl PathInterner {
    /// Get the id corresponding to `path`.
    ///
    /// If `path` does not exists in `self`, returns [`None`].
    pub(crate) fn get(&self, path: &VirtualFsPath) -> Option<FileId> {
        self.map.get_index_of(path).map(|i| FileId(i as u32))
    }

    /// Insert `path` in `self`.
    ///
    /// - If `path` already exists in `self`, returns its associated id;
    /// - Else, returns a newly allocated id.
    pub(crate) fn intern(&mut self, path: VirtualFsPath) -> FileId {
        let (id, _added) = self.map.insert_full(path);
        assert!(id < u32::MAX as usize);
        FileId(id as u32)
    }

    /// Returns the path corresponding to `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not exists in `self`.
    pub(crate) fn lookup(&self, id: FileId) -> &VirtualFsPath {
        self.map.get_index(id.0 as usize).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_interner() {
        let mut interner = PathInterner::default();
        let id1 = interner.intern(VirtualFsPath::new_virtual_path("/foo".to_string()));
        let id2 = interner.intern(VirtualFsPath::new_virtual_path("/bar".to_string()));
        assert_ne!(id1, id2);
        assert_eq!(
            interner.lookup(id1),
            &VirtualFsPath::new_virtual_path("/foo".to_string())
        );
        assert_eq!(
            interner.lookup(id2),
            &VirtualFsPath::new_virtual_path("/bar".to_string())
        );
        assert_eq!(
            interner.get(&VirtualFsPath::new_virtual_path("/foo".to_string())),
            Some(id1)
        );
        assert_eq!(
            interner.get(&VirtualFsPath::new_virtual_path("/bar".to_string())),
            Some(id2)
        );
        assert_eq!(
            interner.get(&VirtualFsPath::new_virtual_path("/baz".to_string())),
            None
        );
    }
}
