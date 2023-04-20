//! Utilities for path manipulation.

use alloc::string::String;

/// Returns the canonical form of the path with all intermediate components
/// normalized.
///
/// It won't force convert the path to an absolute form.
///
/// # Examples
///
/// ```
/// use axfs_vfs::path::canonicalize;
///
/// assert_eq!(canonicalize("/path/./to//foo"), "/path/to/foo");
/// assert_eq!(canonicalize("/./path/to/../bar.rs"), "/path/bar.rs");
/// assert_eq!(canonicalize("./foo/./bar"), "foo/bar");
/// ```
pub fn canonicalize(path: &str) -> String {
    let mut buf = String::new();
    let is_absolute = path.starts_with('/');
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                while !buf.is_empty() {
                    if buf == "/" {
                        break;
                    }
                    let c = buf.pop().unwrap();
                    if c == '/' {
                        break;
                    }
                }
            }
            _ => {
                if buf.is_empty() {
                    if is_absolute {
                        buf.push('/');
                    }
                } else if &buf[buf.len() - 1..] != "/" {
                    buf.push('/');
                }
                buf.push_str(part);
            }
        }
    }
    if is_absolute && buf.is_empty() {
        buf.push('/');
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_canonicalize() {
        assert_eq!(canonicalize(""), "");
        assert_eq!(canonicalize("///"), "/");
        assert_eq!(canonicalize("//a//.//b///c//"), "/a/b/c");
        assert_eq!(canonicalize("/a/../"), "/");
        assert_eq!(canonicalize("/a/../..///"), "/");
        assert_eq!(canonicalize("a/../"), "");
        assert_eq!(canonicalize("a/..//.."), "");
        assert_eq!(canonicalize("././a"), "a");
        assert_eq!(canonicalize(".././a"), "a");
        assert_eq!(canonicalize("/././a"), "/a");
        assert_eq!(canonicalize("/abc/../abc"), "/abc");
        assert_eq!(canonicalize("/test"), "/test");
        assert_eq!(canonicalize("/test/"), "/test");
        assert_eq!(canonicalize("test/"), "test");
        assert_eq!(canonicalize("test"), "test");
        assert_eq!(canonicalize("/test//"), "/test");
        assert_eq!(canonicalize("/test/foo"), "/test/foo");
        assert_eq!(canonicalize("/test/foo/"), "/test/foo");
        assert_eq!(canonicalize("/test/foo/bar"), "/test/foo/bar");
        assert_eq!(canonicalize("/test/foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//./foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//./.foo/bar//"), "/test/.foo/bar");
        assert_eq!(canonicalize("/test//./..foo/bar//"), "/test/..foo/bar");
        assert_eq!(canonicalize("/test//./../foo/bar//"), "/foo/bar");
        assert_eq!(canonicalize("/test/../foo"), "/foo");
        assert_eq!(canonicalize("/test/bar/../foo"), "/test/foo");
        assert_eq!(canonicalize("../foo"), "foo");
        assert_eq!(canonicalize("../foo/"), "foo");
        assert_eq!(canonicalize("/../foo"), "/foo");
        assert_eq!(canonicalize("/../foo/"), "/foo");
        assert_eq!(canonicalize("/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo/.."), "/");
        assert_eq!(canonicalize("/bleh/bar/../../foo/../meh"), "/meh");
    }
}
