use crate::file_id::FileId;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnchoredPathBuf {
    pub anchor: FileId,
    pub path: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnchoredPath<'a> {
    pub anchor: FileId,
    pub path: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchored_path() {
        let path = AnchoredPathBuf {
            anchor: FileId(22),
            path: "foo".to_string(),
        };
        assert_eq!(path.anchor, FileId(22));
        assert_eq!(path.path, "foo");

        let path = AnchoredPath {
            anchor: FileId(22),
            path: "foo",
        };
        assert_eq!(path.anchor, FileId(22));
        assert_eq!(path.path, "foo");
    }

    #[test]
    fn test_anchored_path_eq() {
        let path1 = AnchoredPathBuf {
            anchor: FileId(22),
            path: "foo".to_string(),
        };
        let path2 = AnchoredPathBuf {
            anchor: FileId(22),
            path: "foo".to_string(),
        };
        assert_eq!(path1, path2);

        let path1 = AnchoredPath {
            anchor: FileId(22),
            path: "foo",
        };
        let path2 = AnchoredPath {
            anchor: FileId(22),
            path: "foo",
        };
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_anchored_path_ne() {
        let path1 = AnchoredPathBuf {
            anchor: FileId(22),
            path: "foo".to_string(),
        };
        let path2 = AnchoredPathBuf {
            anchor: FileId(23),
            path: "foo".to_string(),
        };
        assert_ne!(path1, path2);

        let path1 = AnchoredPath {
            anchor: FileId(22),
            path: "foo",
        };
        let path2 = AnchoredPath {
            anchor: FileId(23),
            path: "foo",
        };
        assert_ne!(path1, path2);
    }
}
