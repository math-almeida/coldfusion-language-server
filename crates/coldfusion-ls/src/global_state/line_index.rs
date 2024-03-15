use memchr::memmem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LineEndings {
    Dos,
    Unix,
}

impl LineEndings {
    pub(crate) fn normalize(src: String) -> (String, LineEndings) {
        let mut buf = src.into_bytes();
        let mut gap_len = 0;
        let mut tail = buf.as_mut_slice();
        let mut crlf_seen = false;

        let finder = memmem::Finder::new(b"\r\n");

        loop {
            let idx = match finder.find(&tail[gap_len..]) {
                None if crlf_seen => tail.len(),
                // SAFETY: buf is unchanged and therefore still contains utf8 data
                None => return (unsafe { String::from_utf8_unchecked(buf) }, LineEndings::Unix),
                Some(idx) => {
                    crlf_seen = true;
                    idx + gap_len
                }
            };
            tail.copy_within(gap_len..idx, 0);
            tail = &mut tail[idx - gap_len..];
            if tail.len() == gap_len {
                break;
            }
            gap_len += 1;
        }
        let src = unsafe {
            buf.set_len(buf.len() - gap_len);
            String::from_utf8_unchecked(buf)
        };
        (src, LineEndings::Dos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix() {
        let src = "a\nb\nc\n\n\n\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Unix);
        assert_eq!(res, src);
    }

    #[test]
    fn dos() {
        let src = "\r\na\r\n\r\nb\r\nc\r\n\r\n\r\n\r\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Dos);
        assert_eq!(res, "\na\n\nb\nc\n\n\n\n");
    }

    #[test]
    fn mixed() {
        let src = "a\r\nb\r\nc\r\n\n\r\n\n";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Dos);
        assert_eq!(res, "a\nb\nc\n\n\n\n");
    }

    #[test]
    fn none() {
        let src = "abc";
        let (res, endings) = LineEndings::normalize(src.into());
        assert_eq!(endings, LineEndings::Unix);
        assert_eq!(res, src);
    }
}
