use super::Arena;
use std::ops::{Deref, DerefMut};

/// A `Box` is a pointer to a value that is allocated in an arena.
/// Boxed values implement Deref and DerefMut to allow for dereferencing the
/// pointer to access the value.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Box<T> {
    arena: *const Arena,
    ptr: *mut T,
}

impl<T> Box<T> {
    pub fn new(arena: &Arena) -> Option<Self> {
        arena.alloc::<T>(1).map(|ptr| Box { arena, ptr })
    }

    pub fn from_value(arena: &Arena, value: T) -> Option<Self> {
        let mut boxed = Self::new(arena)?;
        *boxed = value;
        Some(boxed)
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<T: Copy> Clone for Box<T> {
    fn clone(&self) -> Self {
        let arena = unsafe { &*self.arena };
        arena.push(*self.deref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Arena;

    #[test]
    fn test_boxed() {
        let arena = Arena::new(1024);
        let mut boxed = arena.push(42).unwrap();

        assert_eq!(*boxed, 42);
        *boxed = 43;
        assert_eq!(*boxed, 43);
        assert_eq!(arena.occupied(), core::mem::size_of::<i32>());

        let cloned = boxed.clone();
        assert_eq!(*cloned, 43);
        *boxed = 44;

        assert_eq!(*boxed, 44);
        assert_eq!(*cloned, 43);

        assert_eq!(arena.occupied(), core::mem::size_of::<i32>() * 2);
    }

    #[test]
    fn test_boxed_as_ref() {
        let arena = Arena::new(1024);
        let boxed = arena.push(42).unwrap();

        assert_eq!(boxed.as_ref(), &42);
    }

    #[test]
    fn test_boxed_as_mut() {
        let arena = Arena::new(1024);
        let mut boxed = arena.push(42).unwrap();

        assert_eq!(boxed.as_mut(), &mut 42);
    }

    #[test]
    fn test_boxed_as_mut_ptr() {
        let arena = Arena::new(1024);
        let mut boxed = arena.push(42).unwrap();

        let ptr = boxed.as_mut_ptr();
        unsafe {
            *ptr = 43;
        }

        assert_eq!(*boxed, 43);
    }

    #[test]
    fn test_boxed_as_ptr() {
        let arena = Arena::new(1024);
        let boxed = arena.push(42).unwrap();

        let ptr = boxed.as_ptr();
        unsafe {
            assert_eq!(*ptr, 42);
        }
    }

    #[test]
    fn test_boxed_deref() {
        let arena = Arena::new(1024);
        let boxed = arena.push(42).unwrap();

        assert_eq!(*boxed, 42);
    }

    #[test]
    fn test_boxed_deref_mut() {
        let arena = Arena::new(1024);
        let mut boxed = arena.push(42).unwrap();

        *boxed = 43;

        assert_eq!(*boxed, 43);
    }

    #[test]
    fn test_boxed_clone() {
        let arena = Arena::new(1024);
        let mut boxed = arena.push(42).unwrap();
        let cloned = boxed.clone();

        assert_eq!(*cloned, 42);
        assert_eq!(*boxed, 42);

        *boxed = 43;

        assert_eq!(*boxed, 43);
        assert_eq!(*cloned, 42);
        assert_eq!(arena.occupied(), core::mem::size_of::<i32>() * 2);
    }
}
