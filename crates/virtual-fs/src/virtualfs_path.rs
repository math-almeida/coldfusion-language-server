use std::fmt;

use paths::{AbsPath, AbsPathBuf, RelPath};
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct VirtualFsPath(pub VirtualFsPathRepr);

impl VirtualFsPath {
    pub fn new_virtual_path(path: String) -> Self {
        VirtualFsPath(VirtualFsPathRepr::VirtualPath(VirtualPath(path)))
    }

    pub fn new_real_path(path: String) -> Self {
        Self::from(AbsPathBuf::assert(path.into()))
    }

    pub fn as_path(&self) -> Option<&AbsPath> {
        match &self.0 {
            VirtualFsPathRepr::PathBuf(path) => Some(path.as_path()),
            VirtualFsPathRepr::VirtualPath(_) => None,
        }
    }

    pub fn join(&self, path: &str) -> Option<VirtualFsPath> {
        match &self.0 {
            VirtualFsPathRepr::PathBuf(it) => {
                let res = it.join(path).normalize();
                Some(VirtualFsPath(VirtualFsPathRepr::PathBuf(res)))
            }
            VirtualFsPathRepr::VirtualPath(it) => {
                let res = it.join(path)?;
                Some(VirtualFsPath(VirtualFsPathRepr::VirtualPath(res)))
            }
        }
    }

    pub fn pop(&mut self) -> bool {
        match &mut self.0 {
            VirtualFsPathRepr::PathBuf(it) => it.pop(),
            VirtualFsPathRepr::VirtualPath(it) => it.pop(),
        }
    }

    pub fn starts_with(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (VirtualFsPathRepr::PathBuf(lhs), VirtualFsPathRepr::PathBuf(rhs)) => {
                lhs.starts_with(rhs)
            }
            (VirtualFsPathRepr::VirtualPath(lhs), VirtualFsPathRepr::VirtualPath(rhs)) => {
                lhs.starts_with(rhs)
            }
            _ => false,
        }
    }

    pub fn strip_prefix(&self, base: &Self) -> Option<&RelPath> {
        match (&self.0, &base.0) {
            (VirtualFsPathRepr::PathBuf(lhs), VirtualFsPathRepr::PathBuf(rhs)) => {
                lhs.strip_prefix(rhs)
            }
            (VirtualFsPathRepr::VirtualPath(lhs), VirtualFsPathRepr::VirtualPath(rhs)) => {
                lhs.strip_prefix(rhs)
            }
            _ => None,
        }
    }

    pub fn parent(&self) -> Option<VirtualFsPath> {
        let mut parent = self.clone();
        if parent.pop() {
            Some(parent)
        } else {
            None
        }
    }

    pub fn name_and_extension(&self) -> Option<(&str, Option<&str>)> {
        match &self.0 {
            VirtualFsPathRepr::PathBuf(it) => it.name_and_extension(),
            VirtualFsPathRepr::VirtualPath(it) => it.name_and_extension(),
        }
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>) {
        let tag = match &self.0 {
            VirtualFsPathRepr::PathBuf(_) => 0,
            VirtualFsPathRepr::VirtualPath(_) => 1,
        };

        buf.push(tag);
        match &self.0 {
            VirtualFsPathRepr::PathBuf(path) => {
                #[cfg(windows)]
                {
                    use windows_paths::Encode;
                    let path: &std::path::Path = path.as_ref();
                    let components = path.components();
                    let mut add_separator = false;
                    for component in components {
                        if add_separator {
                            windows_paths::SEP.encode(buf);
                        }
                        let len_before = buf.len();
                        match component {
                            std::path::Component::Prefix(prefix) => {
                                prefix.kind().encode(buf);
                            }
                            std::path::Component::RootDir => {
                                if (!add_separator) {
                                    component.as_os_str().encode(buf);
                                }
                            }
                            _ => component.as_os_str().encode(buf),
                        }

                        add_separator = len_before != buf.len();
                    }
                }
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt;
                    buf.extend(path.as_os_str().as_bytes());
                }
                #[cfg(not(any(windows, unix)))]
                {
                    buf.extend(path.as_os_str().to_string_lossy().as_bytes());
                }
            }
            VirtualFsPathRepr::VirtualPath(VirtualPath(s)) => {
                buf.extend(s.as_bytes());
            }
        }
    }
}

#[cfg(windows)]
mod windows_paths {
    pub(crate) trait Encode {
        fn encode(&self, buf: &mut Vec<u8>);
    }

    impl Encode for std::ffi::OsStr {
        fn encode(&self, buf: &mut Vec<u8>) {
            use std::os::windows::ffi::OsStrExt;
            for wchar in self.encode_wide() {
                buf.extend(wchar.to_le_bytes().iter().copied());
            }
        }
    }

    impl Encode for u8 {
        fn encode(&self, buf: &mut Vec<u8>) {
            let wide = *self as u16;
            buf.extend(wide.to_le_bytes().iter().copied())
        }
    }

    impl Encode for &str {
        fn encode(&self, buf: &mut Vec<u8>) {
            debug_assert!(self.is_ascii());
            for b in self.as_bytes() {
                b.encode(buf)
            }
        }
    }

    pub(crate) const SEP: &str = "\\";
    const VERBATIM: &str = "\\\\?\\";
    const UNC: &str = "UNC";
    const DEVICE: &str = "\\\\.\\";
    const COLON: &str = ":";

    impl Encode for std::path::Prefix<'_> {
        fn encode(&self, buf: &mut Vec<u8>) {
            match self {
                std::path::Prefix::Verbatim(c) => {
                    VERBATIM.encode(buf);
                    c.encode(buf);
                }
                std::path::Prefix::VerbatimUNC(server, share) => {
                    VERBATIM.encode(buf);
                    UNC.encode(buf);
                    SEP.encode(buf);
                    server.encode(buf);
                    SEP.encode(buf);
                    share.encode(buf);
                }
                std::path::Prefix::VerbatimDisk(d) => {
                    VERBATIM.encode(buf);
                    d.encode(buf);
                    COLON.encode(buf);
                }
                std::path::Prefix::DeviceNS(device) => {
                    DEVICE.encode(buf);
                    device.encode(buf);
                }
                std::path::Prefix::UNC(server, share) => {
                    SEP.encode(buf);
                    SEP.encode(buf);
                    server.encode(buf);
                    SEP.encode(buf);
                    share.encode(buf);
                }
                std::path::Prefix::Disk(d) => {
                    d.encode(buf);
                    COLON.encode(buf);
                }
            }
        }
    }
    #[test]
    fn paths_encoding() {
        // drive letter casing agnostic
        test_eq("C:/x.rs", "c:/x.rs");
        // separator agnostic
        test_eq("C:/x/y.rs", "C:\\x\\y.rs");

        fn test_eq(a: &str, b: &str) {
            let mut b1 = Vec::new();
            let mut b2 = Vec::new();
            vfs(a).encode(&mut b1);
            vfs(b).encode(&mut b2);
            assert_eq!(b1, b2);
        }
    }

    #[test]
    fn test_sep_root_dir_encoding() {
        let mut buf = Vec::new();
        vfs("C:/x/y").encode(&mut buf);
        assert_eq!(&buf, &[0, 67, 0, 58, 0, 92, 0, 120, 0, 92, 0, 121, 0])
    }

    #[cfg(test)]
    fn vfs(str: &str) -> super::VfsPath {
        use super::{AbsPathBuf, VfsPath};
        VfsPath::from(AbsPathBuf::try_from(str).unwrap())
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum VirtualFsPathRepr {
    VirtualPath(VirtualPath),
    PathBuf(AbsPathBuf),
}

impl From<AbsPathBuf> for VirtualFsPath {
    fn from(v: AbsPathBuf) -> Self {
        VirtualFsPath(VirtualFsPathRepr::PathBuf(v.normalize()))
    }
}

impl fmt::Display for VirtualFsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            VirtualFsPathRepr::PathBuf(it) => it.fmt(f),
            VirtualFsPathRepr::VirtualPath(VirtualPath(it)) => it.fmt(f),
        }
    }
}

impl fmt::Debug for VirtualFsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Debug for VirtualFsPathRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualFsPathRepr::VirtualPath(it) => it.fmt(f),
            VirtualFsPathRepr::PathBuf(it) => it.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct VirtualPath(pub String);

impl VirtualPath {
    /// Returns `true` if `other` is a prefix of `self` (as strings).
    fn starts_with(&self, other: &VirtualPath) -> bool {
        self.0.starts_with(&other.0)
    }

    fn strip_prefix(&self, base: &VirtualPath) -> Option<&RelPath> {
        <_ as AsRef<std::path::Path>>::as_ref(&self.0)
            .strip_prefix(&base.0)
            .ok()
            .map(RelPath::new_unchecked)
    }

    /// Remove the last component of `self`.
    ///
    /// This will find the last `'/'` in `self`, and remove everything after it,
    /// including the `'/'`.
    ///
    /// If `self` contains no `'/'`, returns `false`; else returns `true`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut path = VirtualPath("/foo/bar".to_string());
    /// path.pop();
    /// assert_eq!(path.0, "/foo");
    /// path.pop();
    /// assert_eq!(path.0, "");
    /// ```
    fn pop(&mut self) -> bool {
        let pos = match self.0.rfind('/') {
            Some(pos) => pos,
            None => return false,
        };
        self.0 = self.0[..pos].to_string();
        true
    }

    /// Append the given *relative* path `path` to `self`.
    ///
    /// This will resolve any leading `"../"` in `path` before appending it.
    ///
    /// Returns [`None`] if `path` has more leading `"../"` than the number of
    /// components in `self`.
    ///
    /// # Notes
    ///
    /// In practice, appending here means `self/path` as strings.
    fn join(&self, mut path: &str) -> Option<VirtualPath> {
        let mut res = self.clone();
        while path.starts_with("../") {
            if !res.pop() {
                return None;
            }
            path = &path["../".len()..];
        }
        path = path.trim_start_matches("./");
        res.0 = format!("{}/{path}", res.0);
        Some(res)
    }

    /// Returns `self`'s base name and file extension.
    ///
    /// # Returns
    /// - `None` if `self` ends with `"//"`.
    /// - `Some((name, None))` if `self`'s base contains no `.`, or only one `.` at
    /// the start.
    /// - `Some((name, Some(extension))` else.
    ///
    /// # Note
    /// The extension will not contains `.`. This means `"/foo/bar.baz.rs"` will
    /// return `Some(("bar.baz", Some("rs"))`.
    fn name_and_extension(&self) -> Option<(&str, Option<&str>)> {
        let file_path = if self.0.ends_with('/') {
            &self.0[..&self.0.len() - 1]
        } else {
            &self.0
        };
        let file_name = match file_path.rfind('/') {
            Some(position) => &file_path[position + 1..],
            None => file_path,
        };

        if file_name.is_empty() {
            None
        } else {
            let mut file_stem_and_extension = file_name.rsplitn(2, '.');
            let extension = file_stem_and_extension.next();
            let file_stem = file_stem_and_extension.next();

            match (file_stem, extension) {
                (None, None) => None,
                (None | Some(""), Some(_)) => Some((file_name, None)),
                (Some(file_stem), extension) => Some((file_stem, extension)),
            }
        }
    }
}
