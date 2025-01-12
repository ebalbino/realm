use std::boxed::Box;
use std::cell::Cell;

use super::{ArenaBox, ArenaArray, ArenaString, ArenaTable, ArenaList};

/// An arena is a fixed size memory buffer that can be used to allocate
/// memory for objects that have a lifetime that is bound to the arena.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Arena {
    data: Box<[u8]>,
    offset: Cell<usize>,
    generation: Cell<usize>,
}

impl Arena {
    pub fn new(size: usize) -> Arena {
        Arena {
            data: vec![0; size].into_boxed_slice(),
            offset: Cell::new(0),
            generation: Cell::new(0),
        }
    }

    pub fn alloc<T>(&self, len: usize) -> Option<*mut T> {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        let offset = (self.offset.get() + align - 1) & !(align - 1);
        let new_offset = offset + (size * len);

        if new_offset <= self.size() {
            let ptr = &self.data[offset] as *const u8 as *mut T;
            self.offset.set(new_offset);

            Some(ptr)
        } else {
            None
        }
    }

    pub fn generation(&self) -> usize {
        self.generation.get()
    }

    pub fn make_box<T>(&self) -> Option<ArenaBox<T>> {
        ArenaBox::new(self)
    }

    pub fn make_array<T>(&self, capacity: usize) -> Option<ArenaArray<T>> {
        ArenaArray::new(self, 0, capacity)
    }

    pub fn make_string(&self, capacity: usize) -> Option<ArenaString> {
        ArenaString::new(self, capacity)
    }

    pub fn make_table<T>(&self, capacity: usize) -> Option<ArenaTable<T>> {
        ArenaTable::new(self, capacity)
    }

    pub fn make_list<T>(&self) -> Option<ArenaList<T>> {
        Some(ArenaList::new(self))
    }

    pub fn push<T>(&self, value: T) -> Option<ArenaBox<T>> {
        ArenaBox::from_value(self, value)
    }

    pub fn push_array<T>(&self, values: &[T]) -> Option<ArenaArray<T>> {
        ArenaArray::from_slice(self, values)
    }

    pub fn push_string(&self, str: impl AsRef<str>) -> Option<ArenaString> {
        ArenaString::from_str(self, str.as_ref())
    }

    pub fn reset(&self) {
        let offset = self.offset.get();

        // If we have allocated any memory, increment the generation
        if offset > 0 {
            self.generation.set(self.generation.get() + 1);
        }

        self.offset.set(0);
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn occupied(&self) -> usize {
        self.offset.get()
    }

    pub fn is_full(&self) -> bool {
        self.occupied() == self.size()
    }
}
