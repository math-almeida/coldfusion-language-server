use rustc_hash::FxHashMap;
use std::mem;
use virtual_fs::VirtualFsPath;

#[derive(Default, Clone)]
pub struct MemDocs {
    pub(crate) mem_docs: FxHashMap<VirtualFsPath, DocumentData>,
    added_or_removed: bool,
}

impl MemDocs {
    pub(crate) fn contains(&self, path: &VirtualFsPath) -> bool {
        self.mem_docs.contains_key(path)
    }

    pub(crate) fn insert(&mut self, path: VirtualFsPath, data: DocumentData) -> Result<(), ()> {
        self.added_or_removed = true;
        match self.mem_docs.insert(path, data) {
            Some(_) => Err(()),
            None => Ok(()),
        }
    }

    pub(crate) fn remove(&mut self, path: &VirtualFsPath) -> Result<(), ()> {
        self.added_or_removed = true;
        match self.mem_docs.remove(path) {
            Some(_) => Ok(()),
            None => Err(()),
        }
    }

    pub(crate) fn get(&self, path: &VirtualFsPath) -> Option<&DocumentData> {
        self.mem_docs.get(path)
    }

    pub(crate) fn get_mut(&mut self, path: &VirtualFsPath) -> Option<&mut DocumentData> {
        self.mem_docs.get_mut(path)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &VirtualFsPath> {
        self.mem_docs.keys()
    }

    pub(crate) fn take_changes(&mut self) -> bool {
        mem::replace(&mut self.added_or_removed, false)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DocumentData {
    pub(crate) version: i32,
    pub(crate) data: Vec<u8>,
}

impl DocumentData {
    pub(crate) fn new(version: i32, data: Vec<u8>) -> Self {
        DocumentData { version, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_fs::{VirtualFsPathRepr, VirtualPath};

    #[test]
    fn test_mem_docs() {
        let mut mem_docs = MemDocs::default();
        let path = VirtualFsPath(VirtualFsPathRepr::VirtualPath(VirtualPath(
            "test".to_string(),
        )));
        let data = DocumentData::new(0, vec![]);
        mem_docs.insert(path.clone(), data.clone());
        assert!(mem_docs.get(&path).is_some());
        assert_eq!(mem_docs.take_changes(), true);
        assert_eq!(mem_docs.take_changes(), false);
        mem_docs.remove(&path);
        assert!(mem_docs.get(&path).is_none());
        assert_eq!(mem_docs.take_changes(), true);
    }
}
