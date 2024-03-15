mod anchored_path;
mod file_id;
use file_id::FileId;
mod path_interner;
use path_interner::PathInterner;

mod virtualfs_path;
use virtualfs_path::VirtualFsPath;
use std::{fmt, mem};

pub use paths::AbsPathBuf;

#[derive(Default)]
pub struct VirtualFS {
    data: Vec<FileState>,
    changes: Vec<ChangedFile>,
    interner: PathInterner,
}

#[derive(Copy, PartialEq, PartialOrd, Clone)]
pub enum FileState {
    Exists,
    Deleted,
}

#[derive(Debug)]
pub struct ChangedFile {
    pub file_id: FileId,
    pub change: Change,
}

impl ChangedFile {
    pub fn exists(&self) -> bool {
        !matches!(self.change, Change::Delete)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Change {
    Create(Vec<u8>),
    Modify(Vec<u8>),
    Delete,
}

#[derive(Eq, PartialEq, Debug)]
pub enum ChangeKind {
    Create,
    Modify,
    Delete,
}

impl VirtualFS {
    pub fn file_id(&self, path: &VirtualFsPath) -> Option<FileId> {
        self.interner
            .get(path)
            .filter(|&it| matches!(self.get(it), FileState::Exists))
    }

    pub fn file_path(&self, file_id: FileId) -> &VirtualFsPath {
        self.interner.lookup(file_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FileId, &VirtualFsPath)> + '_ {
        (0..self.data.len())
            .map(|it| FileId(it as u32))
            .filter(move |&file_id| matches!(self.get(file_id), FileState::Exists))
            .map(move |file_id| {
                let path = self.interner.lookup(file_id);
                (file_id, path)
            })
    }

    pub fn set_file_contents(&mut self, path: VirtualFsPath, contents: Option<Vec<u8>>) -> bool {
        let file_id = self.alloc_file_id(path);
        let change_kind = match (self.get(file_id), contents) {
            (FileState::Deleted, None) => return false,
            (FileState::Deleted, Some(v)) => Change::Create(v),
            (FileState::Exists, None) => Change::Delete,
            (FileState::Exists, Some(v)) => Change::Modify(v),
        };
        let changed_file = ChangedFile {
            file_id,
            change: change_kind,
        };

        self.data[file_id.0 as usize] = if changed_file.exists() {
            FileState::Exists
        } else {
            FileState::Deleted
        };
        self.changes.push(changed_file);
        true
    }

    pub fn take_changes(&mut self) -> Vec<ChangedFile> {
        mem::take(&mut self.changes)
    }

    fn alloc_file_id(&mut self, path: VirtualFsPath) -> FileId {
        let file_id = self.interner.intern(path);
        let idx = file_id.0 as usize;
        let len = self.data.len().max(idx + 1);
        self.data.resize(len, FileState::Deleted);
        file_id
    }

    fn get(&self, file_id: FileId) -> FileState {
        self.data[file_id.0 as usize]
    }
}

impl fmt::Debug for VirtualFS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VirtualFS")
            .field("changes", &self.changes)
            .field("n_files", &self.data.len())
            .finish()
    }
}
