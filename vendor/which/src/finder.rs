use crate::checker::CompositeChecker;
use crate::error::*;
#[cfg(windows)]
use crate::helper::has_executable_extension;
use either::Either;
#[cfg(feature = "regex")]
use regex::Regex;
#[cfg(feature = "regex")]
use std::borrow::Borrow;
use std::env;
use std::ffi::OsStr;
#[cfg(any(feature = "regex", target_os = "windows"))]
use std::fs;
use std::iter;
use std::path::{Path, PathBuf};

pub trait Checker {
    fn is_valid(&self, path: &Path) -> bool;
}

trait PathExt {
    fn has_separator(&self) -> bool;

    fn to_absolute<P>(self, cwd: P) -> PathBuf
    where
        P: AsRef<Path>;
}

impl PathExt for PathBuf {
    fn has_separator(&self) -> bool {
        self.components().count() > 1
    }

    fn to_absolute<P>(self, cwd: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        if self.is_absolute() {
            self
        } else {
            let mut new_path = PathBuf::from(cwd.as_ref());
            new_path.push(self);
            new_path
        }
    }
}

pub struct Finder;

impl Finder {
    pub fn new() -> Finder {
        Finder
    }

    pub fn find<T, U, V>(
        &self,
        binary_name: T,
        paths: Option<U>,
        cwd: Option<V>,
        binary_checker: CompositeChecker,
    ) -> Result<impl Iterator<Item = PathBuf>>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
        V: AsRef<Path>,
    {
        let path = PathBuf::from(&binary_name);

        let binary_path_candidates = match cwd {
            Some(cwd) if path.has_separator() => {
                // Search binary in cwd if the path have a path separator.
                Either::Left(Self::cwd_search_candidates(path, cwd).into_iter())
            }
            _ => {
                // Search binary in PATHs(defined in environment variable).
                let p = paths.ok_or(Error::CannotFindBinaryPath)?;
                let paths: Vec<_> = env::split_paths(&p).collect();

                Either::Right(Self::path_search_candidates(path, paths).into_iter())
            }
        };

        Ok(binary_path_candidates
            .filter(move |p| binary_checker.is_valid(p))
            .map(correct_casing))
    }

    #[cfg(feature = "regex")]
    pub fn find_re<T>(
        &self,
        binary_regex: impl Borrow<Regex>,
        paths: Option<T>,
        binary_checker: CompositeChecker,
    ) -> Result<impl Iterator<Item = PathBuf>>
    where
        T: AsRef<OsStr>,
    {
        let p = paths.ok_or(Error::CannotFindBinaryPath)?;
        // Collect needs to happen in order to not have to
        // change the API to borrow on `paths`.
        #[allow(clippy::needless_collect)]
        let paths: Vec<_> = env::split_paths(&p).collect();

        let matching_re = paths
            .into_iter()
            .flat_map(fs::read_dir)
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(move |p| {
                if let Some(unicode_file_name) = p.file_name().unwrap().to_str() {
                    binary_regex.borrow().is_match(unicode_file_name)
                } else {
                    false
                }
            })
            .filter(move |p| binary_checker.is_valid(p));

        Ok(matching_re)
    }

    fn cwd_search_candidates<C>(binary_name: PathBuf, cwd: C) -> impl IntoIterator<Item = PathBuf>
    where
        C: AsRef<Path>,
    {
        let path = binary_name.to_absolute(cwd);

        Self::append_extension(iter::once(path))
    }

    fn path_search_candidates<P>(
        binary_name: PathBuf,
        paths: P,
    ) -> impl IntoIterator<Item = PathBuf>
    where
        P: IntoIterator<Item = PathBuf>,
    {
        let new_paths = paths.into_iter().map(move |p| p.join(binary_name.clone()));

        Self::append_extension(new_paths)
    }

    #[cfg(unix)]
    fn append_extension<P>(paths: P) -> impl IntoIterator<Item = PathBuf>
    where
        P: IntoIterator<Item = PathBuf>,
    {
        paths
    }

    #[cfg(windows)]
    fn append_extension<P>(paths: P) -> impl IntoIterator<Item = PathBuf>
    where
        P: IntoIterator<Item = PathBuf>,
    {
        use once_cell::sync::Lazy;

        // Sample %PATHEXT%: .COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC
        // PATH_EXTENSIONS is then [".COM", ".EXE", ".BAT", …].
        // (In one use of PATH_EXTENSIONS we skip the dot, but in the other we need it;
        // hence its retention.)
        static PATH_EXTENSIONS: Lazy<Vec<String>> = Lazy::new(|| {
            env::var("PATHEXT")
                .map(|pathext| {
                    pathext
                        .split(';')
                        .filter_map(|s| {
                            if s.as_bytes().first() == Some(&b'.') {
                                Some(s.to_owned())
                            } else {
                                // Invalid segment; just ignore it.
                                None
                            }
                        })
                        .collect()
                })
                // PATHEXT not being set or not being a proper Unicode string is exceedingly
                // improbable and would probably break Windows badly. Still, don't crash:
                .unwrap_or_default()
        });

        paths
            .into_iter()
            .flat_map(move |p| -> Box<dyn Iterator<Item = _>> {
                // Check if path already have executable extension
                if has_executable_extension(&p, &PATH_EXTENSIONS) {
                    Box::new(iter::once(p))
                } else {
                    let bare_file = p.extension().map(|_| p.clone());
                    // Appended paths with windows executable extensions.
                    // e.g. path `c:/windows/bin[.ext]` will expand to:
                    // [c:/windows/bin.ext]
                    // c:/windows/bin[.ext].COM
                    // c:/windows/bin[.ext].EXE
                    // c:/windows/bin[.ext].CMD
                    // ...
                    Box::new(
                        bare_file
                            .into_iter()
                            .chain(PATH_EXTENSIONS.iter().map(move |e| {
                                // Append the extension.
                                let mut p = p.clone().into_os_string();
                                p.push(e);

                                PathBuf::from(p)
                            })),
                    )
                }
            })
    }
}

#[cfg(target_os = "windows")]
fn correct_casing(mut p: PathBuf) -> PathBuf {
    if let (Some(parent), Some(file_name)) = (p.parent(), p.file_name()) {
        if let Ok(iter) = fs::read_dir(parent) {
            for e in iter.filter_map(std::result::Result::ok) {
                if e.file_name().eq_ignore_ascii_case(file_name) {
                    p.pop();
                    p.push(e.file_name());
                    break;
                }
            }
        }
    }
    p
}

#[cfg(not(target_os = "windows"))]
fn correct_casing(p: PathBuf) -> PathBuf {
    p
}
