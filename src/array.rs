use super::Arena;
use std::ops::{Deref, DerefMut};

/// A fixed-size array that is allocated in an arena.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Array<T> {
    arena: *const Arena,
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Array<T> {
    pub fn new(arena: &Arena, len: usize, capacity: usize) -> Option<Self> {
        if capacity < len {
            return None;
        }

        let ptr = arena.alloc::<T>(capacity)?;

        Some(Array {
            arena,
            ptr,
            len,
            capacity,
        })
    }

    pub fn from_slice(arena: &Arena, slice: &[T]) -> Option<Self> {
        let len = slice.len();
        let mut array = Self::new(arena, 0, len)?;

        array.concat(slice)?;

        Some(array)
    }

    pub fn push(&mut self, value: T) {
        let len = self.len;

        if self.capacity > len {
            let inner = self.deref_mut();
            inner[len] = value;
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<&T> {
        let len = self.len;

        if len > 0 {
            self.len -= 1;
            Some(&self.deref_mut()[len - 1])
        } else {
            None
        }
    }

    pub fn concat(&mut self, values: &[T]) -> Option<usize> {
        let len = self.len();
        let new_len = len + values.len();

        if new_len <= self.capacity() {
            let ptr = self.as_mut_ptr();

            unsafe {
                core::ptr::copy_nonoverlapping(values.as_ptr(), ptr.add(len), values.len());
            }

            self.len = new_len;
            Some(new_len)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr, self.capacity) }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.capacity) }
    }
}

impl<T> AsRef<[T]> for Array<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T> AsMut<[T]> for Array<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T> Clone for Array<T> {
    fn clone(&self) -> Self {
        let new_ptr = unsafe { (*self.arena).push_array(&self[..]).unwrap().as_ptr() as *mut T };

        Array {
            arena: self.arena,
            ptr: new_ptr,
            len: self.len,
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::Arena;

    #[test]
    fn test_array() {
        let arena = Arena::new(1024);
        let array: Option<Array<i32>> = Array::new(&arena, 0, 5);

        assert!(array.is_some());

        let array = array.unwrap();

        assert_eq!(array.len(), 0);
        assert_eq!(array.capacity(), 5);
    }

    #[test]
    fn test_invalid_array() {
        let arena = Arena::new(1024);
        let array: Option<Array<i32>> = Array::new(&arena, 5, 0);

        assert_eq!(array, None);
    }

    #[test]
    fn test_array_from_slice() {
        let arena = Arena::new(1024);
        let array = Array::from_slice(&arena, &[1, 2, 3, 4, 5]).unwrap();

        assert_eq!(array.len(), 5);
        assert_eq!(array.capacity(), 5);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
        assert_eq!(array[4], 5);
    }

    #[test]
    fn test_array_clone() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 5).unwrap();

        for i in 1..6 {
            array.push(i);
        }

        let clone = array.clone();

        assert_eq!(array.len(), 5);
        assert_eq!(array.capacity(), 5);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
        assert_eq!(array[4], 5);

        assert_eq!(clone.len(), 5);
        assert_eq!(clone.capacity(), 5);
        assert_eq!(clone[0], 1);
        assert_eq!(clone[1], 2);
        assert_eq!(clone[2], 3);
        assert_eq!(clone[3], 4);
        assert_eq!(clone[4], 5);
    }

    #[test]
    fn test_array_push() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        assert_eq!(array.len(), 3);
        assert_eq!(array.capacity(), 10);
        assert_eq!(array[0], 6);
        assert_eq!(array[1], 7);
        assert_eq!(array[2], 8);
    }

    #[test]
    fn test_array_pop() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        assert_eq!(array.pop(), Some(&8));
        assert_eq!(array.pop(), Some(&7));
        assert_eq!(array.pop(), Some(&6));
        assert_eq!(array.pop(), None);
    }

    #[test]
    fn test_array_clear() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        assert_eq!(array.len(), 3);
        assert_eq!(array.capacity(), 10);

        array.clear();

        assert_eq!(array.len(), 0);
        assert_eq!(array.capacity(), 10);
    }

    #[test]
    fn test_array_concat() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        assert_eq!(array.len(), 3);
        assert_eq!(array.capacity(), 10);

        let values = [9, 10, 11];
        let result = array.concat(&values);

        assert_eq!(result, Some(6));
        assert_eq!(array.len(), 6);
        assert_eq!(array.capacity(), 10);
        assert_eq!(array[0], 6);
        assert_eq!(array[1], 7);
        assert_eq!(array[2], 8);
        assert_eq!(array[3], 9);
        assert_eq!(array[4], 10);
        assert_eq!(array[5], 11);

        let values = [12, 13, 14, 15, 16, 17, 18, 19];
        let result = array.concat(&values);

        assert_eq!(result, None);
    }

    #[test]
    fn test_array_as_ref() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        let slice = &array;

        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 6);
        assert_eq!(slice[1], 7);
        assert_eq!(slice[2], 8);

        let func = |array: &[i32]| {
            assert_eq!(array.len(), 3);
            assert_eq!(array[0], 6);
            assert_eq!(array[1], 7);
            assert_eq!(array[2], 8);
        };

        func(array.as_ref());
    }

    #[test]
    fn test_array_as_mut() {
        let arena = Arena::new(1024);
        let mut array = Array::new(&arena, 0, 10).unwrap();

        array.push(6);
        array.push(7);
        array.push(8);

        let func = |array: &mut [i32]| {
            array[0] += 1;
            array[1] += 1;
            array[2] += 1;
        };

        func(array.as_mut());

        assert_eq!(array.len(), 3);
        assert_eq!(array[0], 7);
        assert_eq!(array[1], 8);
        assert_eq!(array[2], 9);
    }
}
