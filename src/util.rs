use std::ffi::{CString, AsOsStr, OsStr, OsString};
use std::iter::IntoIterator;
use std::path::{Path, PathBuf};
use libc::{c_char, size_t};

use {raw, Error};

#[doc(hidden)]
pub trait Binding: Sized {
    type Raw;

    unsafe fn from_raw(raw: Self::Raw) -> Self;
    fn raw(&self) -> Self::Raw;

    unsafe fn from_raw_opt<T>(raw: T) -> Option<Self>
        where T: PtrExt + Copy, Self: Binding<Raw=T>
    {
        if raw.is_null() {
            None
        } else {
            Some(Binding::from_raw(raw))
        }
    }
}

pub fn iter2cstrs<T, I>(iter: I) -> Result<(Vec<CString>, Vec<*const c_char>,
                                            raw::git_strarray), Error>
    where T: IntoCString, I: IntoIterator<Item=T>
{
    let cstrs: Vec<_> = try!(iter.into_iter().map(|i| i.into_c_string()).collect());
    let ptrs = cstrs.iter().map(|i| i.as_ptr()).collect::<Vec<_>>();
    let raw = raw::git_strarray {
        strings: ptrs.as_ptr() as *mut _,
        count: ptrs.len() as size_t,
    };
    Ok((cstrs, ptrs, raw))
}

#[cfg(unix)]
pub fn bytes2path(b: &[u8]) -> &Path {
    use std::os::unix::prelude::*;
    Path::new(<OsStr as OsStrExt>::from_bytes(b))
}
#[cfg(windows)]
pub fn bytes2path(b: &[u8]) -> &Path {
    use std::str;
    Path::new(str::from_utf8(b).unwrap())
}

/// A class of types that can be converted to C strings.
///
/// These types are represented internally as byte slices and it is quite rare
/// for them to contain an interior 0 byte.
pub trait IntoCString {
    /// Consume this container, converting it into a CString
    fn into_c_string(self) -> Result<CString, Error>;
}

impl<'a, T: IntoCString + Clone> IntoCString for &'a T {
    fn into_c_string(self) -> Result<CString, Error> {
        self.clone().into_c_string()
    }
}

impl<'a> IntoCString for &'a str {
    fn into_c_string(self) -> Result<CString, Error> {
        Ok(try!(CString::new(self)))
    }
}

impl IntoCString for String {
    fn into_c_string(self) -> Result<CString, Error> {
        Ok(try!(CString::new(self.into_bytes())))
    }
}

impl IntoCString for CString {
    fn into_c_string(self) -> Result<CString, Error> { Ok(self) }
}

impl<'a> IntoCString for &'a Path {
    fn into_c_string(self) -> Result<CString, Error> {
        self.as_os_str().into_c_string()
    }
}

impl IntoCString for PathBuf {
    fn into_c_string(self) -> Result<CString, Error> {
        self.as_os_str().into_c_string()
    }
}

impl<'a> IntoCString for &'a OsStr {
    fn into_c_string(self) -> Result<CString, Error> {
        self.to_os_string().into_c_string()
    }
}

impl IntoCString for OsString {
    #[cfg(unix)]
    fn into_c_string(self) -> Result<CString, Error> {
        use std::os::unix::OsStrExt;
        Ok(try!(CString::new(self.as_os_str().as_bytes())))
    }
    #[cfg(windows)]
    fn into_c_string(self) -> Result<CString, Error> {
        match self.to_str() {
            Some(s) => s.into_c_string(),
            None => Error::from_str("only valid unicode paths are accepted \
                                     on windows"),
        }
    }
}

impl<'a> IntoCString for &'a [u8] {
    fn into_c_string(self) -> Result<CString, Error> {
        Ok(try!(CString::new(self)))
    }
}

impl IntoCString for Vec<u8> {
    fn into_c_string(self) -> Result<CString, Error> {
        Ok(try!(CString::new(self)))
    }
}
