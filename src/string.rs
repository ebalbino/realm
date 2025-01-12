use super::{Arena, ArenaArray};
use std::fmt::{Debug, Display};
use std::cmp::Ordering;
use std::fmt::Write;
use std::ops::Deref;

/// An arena backed string. This is a thin wrapper around an `ArenaArray<u8>`.
/// This is a zero-copy string, and is not null-terminated.
/// ArenaString derefs to a `str` and can be used in most places where a `str` is expected.
#[derive(Clone, Eq)]
pub struct ArenaString {
    inner: ArenaArray<u8>,
}

impl Deref for ArenaString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }
}

impl AsRef<str> for ArenaString {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.inner[0..self.inner.len()]) }
    }
}

impl PartialEq for ArenaString {
    fn eq(&self, other: &Self) -> bool {
        self.deref()[0..self.len()] == other.deref()[0..other.len()]
    }
}

impl PartialOrd for ArenaString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArenaString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref()[0..self.len()].cmp(&other.deref()[0..other.len()])
    }
}

impl PartialEq<str> for ArenaString {
    fn eq(&self, other: &str) -> bool {
        &self.deref()[0..self.len()] == other
    }
}

impl Write for ArenaString {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self.concat(s) {
            Some(_) => Ok(()),
            None => Err(std::fmt::Error),
        }
    }
}

impl Debug for ArenaString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl Display for ArenaString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl ArenaString {
    pub fn new(arena: &Arena, capacity: usize) -> Option<Self> {
        ArenaArray::new(arena, 0, capacity).map(|inner| ArenaString { inner })
    }

    pub fn from_slice(arena: &Arena, slice: &[u8]) -> Option<Self> {
        ArenaArray::from_slice(arena, slice).map(|inner| ArenaString { inner })
    }

    pub fn from_str(arena: &Arena, str: &str) -> Option<Self> {
        ArenaArray::from_slice(arena, str.as_bytes()).map(|inner| ArenaString { inner })
    }

    pub fn from_array(inner: ArenaArray<u8>) -> Self {
        ArenaString { inner }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn concat(&mut self, str: &str) -> Option<usize> {
        self.inner.concat(str.as_bytes())
    }

    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;
    use std::fmt::Write;

    #[test]
    fn test_arena_string() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(1024).unwrap();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);
        assert_eq!(string.is_empty(), true);

        let _ = write!(&mut string, "Hello, world!");

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");
        assert_eq!(string.is_empty(), false);

        string.clear();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);
        assert_eq!(string.is_empty(), true);
    }

    #[test]
    fn test_arena_string_concat() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(1024).unwrap();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);

        string.concat("Hello, ").unwrap();
        string.concat("world!").unwrap();

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");

        string.clear();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);
    }

    #[test]
    fn test_arena_string_write() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(1024).unwrap();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);

        let _ = write!(&mut string, "Hello, ").unwrap();
        let _ = write!(&mut string, "world!").unwrap();

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");

        string.clear();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);
    }

    #[test]
    fn test_arena_string_clone() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(128).unwrap();

        let _ = write!(&mut string, "Hello, world!");

        let clone = string.clone();

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");

        assert_eq!(clone.len(), 13);
        assert_eq!(&clone, "Hello, world!");
    }

    #[test]
    fn test_arena_string_eq() {
        let arena = Arena::new(1024);
        let mut string1 = arena.make_string(128).unwrap();
        let mut string2 = arena.make_string(128).unwrap();

        let _ = write!(&mut string1, "Hello, world!");
        let _ = write!(&mut string2, "Hello, world!");

        assert_eq!(string1, string2);
    }

    #[test]
    fn test_arena_string_ord() {
        let arena = Arena::new(1024);
        let mut string1 = arena.make_string(128).unwrap();
        let mut string2 = arena.make_string(128).unwrap();
        let mut string3 = arena.make_string(128).unwrap();

        let _ = write!(&mut string1, "Hello, world!");
        let _ = write!(&mut string2, "Hello, sailor!");
        let _ = write!(&mut string3, "Hello, world!");

        assert_eq!(string1.cmp(&string2), std::cmp::Ordering::Greater);
        assert_eq!(string1.cmp(&string3), std::cmp::Ordering::Equal);
        assert_eq!(string2.cmp(&string3), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_arena_string_as_ref() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(128).unwrap();

        let _ = write!(&mut string, "Hello, world!");

        assert_eq!(&string.as_ref()[0..string.len()], "Hello, world!");
    }

    #[test]
    fn test_arena_string_from_slice() {
        let arena = Arena::new(1024);
        let slice = b"Hello, world!";
        let string = ArenaString::from_slice(&arena, slice).unwrap();

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");
    }

    #[test]
    fn test_arena_string_eq_str() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(128).unwrap();

        let _ = write!(&mut string, "Hello, world!");

        assert_eq!(&string == "Hello, world!", true);
    }

    #[test]
    fn test_arena_string_partial_ord() {
        let arena = Arena::new(1024);
        let mut string1 = arena.make_string(128).unwrap();
        let mut string2 = arena.make_string(128).unwrap();
        let mut string3 = arena.make_string(128).unwrap();

        let _ = write!(&mut string1, "Hello, world!");
        let _ = write!(&mut string2, "Hello, sailor!");
        let _ = write!(&mut string3, "Hello, world!");

        assert_eq!(string1.partial_cmp(&string2), Some(std::cmp::Ordering::Greater));
        assert_eq!(string1.partial_cmp(&string3), Some(std::cmp::Ordering::Equal));
        assert_eq!(string2.partial_cmp(&string3), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_arena_string_write_error() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(20).unwrap();

        assert_eq!(write!(&mut string, "Hello, world!").is_ok(), true);
        assert_eq!(write!(&mut string, "Hello, world!").is_err(), true);
    }

    #[test]
    fn test_arena_string_print() {
        let arena = Arena::new(1024);
        let mut string = arena.make_string(128).unwrap();

        let _ = write!(&mut string, "Hello, world!");

        assert_eq!(format!("{}", string), "Hello, world!");
        assert_eq!(format!("{:?}", string), "\"Hello, world!\"");
    }

    #[test]
    fn test_arena_string_from_array() {
        let arena = Arena::new(1024);
        let array = ArenaArray::from_slice(&arena, b"Hello, world!").unwrap();
        let string = ArenaString::from_array(array);

        assert_eq!(string.len(), 13);
        assert_eq!(&string, "Hello, world!");
    }
}
