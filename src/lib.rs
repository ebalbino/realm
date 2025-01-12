
mod arena;
mod array;
mod boxed;
mod list;
mod string;
mod table;

pub use arena::Arena;
pub use array::Array as ArenaArray;
pub use boxed::Box as ArenaBox;
pub use list::List as ArenaList;
pub use string::ArenaString;
pub use table::{ArenaTable, Key};

#[macro_export]
macro_rules! arena_alloc {
    ($arena:expr) => {
        $arena.make_box()
    };
    ($arena:expr, $value:expr) => {
        $arena.push($value)
    };
}

#[macro_export]
macro_rules! arena_array {
    ($arena:expr, $capacity:expr) => {
        $arena.make_array($capacity)
    };
    ($arena:expr, $values:expr) => {
        $arena.make_array(0).and_then(|mut array| {
            for value in $values.iter() {
                array.push(*value);
            }

            Some(array)
        })
    };
    ($arena:expr, $capacity:expr, $values:expr) => {
        $arena.make_array($capacity).and_then(|mut array| {
            for value in $values.iter() {
                array.push(*value);
            }

            Some(array)
        })
    };
}

#[macro_export]
macro_rules! arena_string {
    ($arena:expr, $capacity:expr) => {
        $arena.make_string($capacity)
    };
    ($arena:expr, $str:expr) => {
        $arena.push_string($str)
    };
    ($arena:expr, $capacity:expr, $str:expr) => {
        $arena.make_string($capacity).and_then(|mut string| {
            let _ = write!(&mut string, "{}", $str);
            Some(string)
        })
    };
}

#[macro_export]
macro_rules! arena_table {
    ($arena:expr, $capacity:expr) => {
        $arena.make_table($capacity)
    };
    ($arena:expr, $capacity:expr, $values:expr) => {
        $arena.make_table($capacity).and_then(|mut table| {
            for (key, value) in $values.iter() {
                table.insert(key, *value);
            }

            Some(table)
        })
    };
}

#[macro_export]
macro_rules! arena_list {
    ($arena:expr) => {
        $arena.make_list()
    };
    ($arena:expr, $values:expr) => {
        $arena.make_list().and_then(|mut list| {
            for value in $values.iter() {
                list.push(*value);
            }

            Some(list)
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn test_arena() {
        let arena = Arena::new(1024);

        assert_eq!(arena.size(), 1024);
        assert_eq!(arena.occupied(), 0);
        assert!(!arena.is_full());

        let _: ArenaBox<i32> = arena_alloc!(arena, 42).unwrap();

        assert_eq!(arena.occupied(), core::mem::size_of::<i32>());

        let _: ArenaBox<i32> = arena_alloc!(arena, 43).unwrap();

        assert_eq!(arena.occupied(), core::mem::size_of::<i32>() * 2);

        arena.clear();
        assert_eq!(arena.occupied(), 0);
    }

    #[test]
    fn test_full_arena() {
        let arena = Arena::new(1024);

        for _ in 0..(1024 / core::mem::size_of::<i32>()) {
            let _: ArenaBox<i32> = arena_alloc!(arena, 42).unwrap();
        }

        assert_eq!(arena.occupied(), 1024);
        assert!(arena.is_full());

        let _: Option<ArenaBox<i32>> = arena_alloc!(arena, 42);
        assert_eq!(arena.occupied(), 1024);
    }

    #[test]
    fn test_arena_box() {
        let arena = Arena::new(1024);
        let mut box1: ArenaBox<i32> = arena_alloc!(arena, 0).unwrap();
        let mut box2: ArenaBox<i32> = arena_alloc!(arena).unwrap();

        *box1 = 42;
        *box2 = 43;

        assert_eq!(*box1, 42);
        assert_eq!(*box2, 43);
    }

    #[test]
    fn test_arena_array() {
        let arena = Arena::new(1024);
        let mut array: ArenaArray<i32> = arena_array!(arena, 10).unwrap();

        for i in 0..10 {
            array[i] = i as i32;
        }

        for i in 0..10 {
            assert_eq!(array[i], i as i32);
        }
    }

    #[test]
    fn test_arena_string() {
        let arena = Arena::new(1024);
        let mut string = arena_string!(arena, 1024).unwrap();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);

        let _ = write!(&mut string, "Hello, world!");

        assert_eq!(string.len(), 13);
        assert_eq!(string.as_str(), "Hello, world!");

        string.clear();

        assert_eq!(string.len(), 0);
        assert_eq!(string.capacity(), 1024);
    }

    #[test]
    fn test_arena_table() {
        let arena = Arena::new(1024);
        let keys = [
            "key-0", "key-1", "key-2", "key-3", "key-4", "key-5", "key-6", "key-7", "key-8",
            "key-9",
        ];
        let mut table = arena_table!(arena, 10).unwrap();

        for i in 0..10 {
            table.insert(&keys[i], i as u8);
        }

        for i in 0..10 {
            assert_eq!(table.get(&keys[i]), Some(&(i as u8)));
        }
    }

    #[test]
    fn test_arena_list() {
        let arena = Arena::new(1024);
        let list = arena_list!(arena, &[42, 43, 44]).unwrap();
        let mut iter = list.iter();

        for i in 0..3 {
            assert_eq!(iter.next(), Some(&(42 + i)));
        }

        assert_eq!(iter.next(), None);
    }
}
