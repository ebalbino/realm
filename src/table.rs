use super::{Arena, ArenaArray, ArenaString};
use fxhash::hash;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Key {
    key: ArenaString,
    hash: usize,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ArenaTable<V> {
    arena: *const Arena,
    keys: ArenaArray<Key>,
    values: ArenaArray<V>,
}

impl<V> ArenaTable<V> {
    pub fn new(arena: &Arena, capacity: usize) -> Option<Self> {
        let keys = arena.make_array(capacity)?;
        let values = arena.make_array(capacity)?;

        Some(Self {
            arena,
            keys,
            values,
        })
    }

    pub fn capacity(&self) -> usize {
        self.keys.capacity()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn get_index(&self, key: &str) -> Option<usize> {
        let hash = hash(key);

        for (i, k) in self.keys.iter().enumerate() {
            if hash == k.hash {
                return Some(i);
            }
        }

        None
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        match self.get_index(key) {
            Some(i) => Some(&self.values[i]),
            None => None,
        }
    }

    pub fn get_key_value(&self, key: &str) -> Option<(&str, &V)> {
        self.get_index(key)
            .map(|i| (&*self.keys[i].key, &self.values[i]))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        match self.get_index(key) {
            Some(i) => Some(&mut self.values[i]),
            None => None,
        }
    }

    pub fn insert(&mut self, key: &str, value: V) -> bool {
        if self.capacity() > self.len() {
            let hash = hash(key);
            let arena = unsafe { &*self.arena };
            let string = arena.push_string(key).unwrap();
            self.keys.push(Key { key: string, hash });
            self.values.push(value);
            return true;
        }

        false
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.get_index(key).is_some()
    }

    pub fn keys(&self) -> &ArenaArray<Key> {
        &self.keys
    }

    pub fn values(&self) -> &ArenaArray<V> {
        &self.values
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.keys
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (&*k.key, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut V)> {
        self.keys
            .iter()
            .zip(self.values.iter_mut())
            .map(|(k, v)| (&*k.key, v))
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::ArenaTable;
    use crate::Arena;

    #[test]
    fn test_table_get() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 10).unwrap();

        assert_eq!(table.capacity(), 10);
        assert_eq!(table.len(), 0);

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);
        assert_eq!(table.insert(&"baz", 44), true);

        assert_eq!(table.len(), 3);

        assert_eq!(table.get(&"foo"), Some(&42));
        assert_eq!(table.get(&"bar"), Some(&43));
        assert_eq!(table.get(&"baz"), Some(&44));
    }

    #[test]
    fn test_table_get_mut() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        let foo = table.get_mut(&"foo").unwrap();
        *foo = 100;

        let bar = table.get_mut(&"bar").unwrap();
        *bar = 200;

        assert_eq!(table.get(&"foo"), Some(&100));
        assert_eq!(table.get(&"bar"), Some(&200));
        assert_eq!(table.get(&"baz"), None);
    }

    #[test]
    fn test_table_get_key_value() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        let (key, value) = table.get_key_value(&"foo").unwrap();
        assert_eq!(key, "foo");
        assert_eq!(value, &42);

        let (key, value) = table.get_key_value(&"bar").unwrap();
        assert_eq!(key, "bar");
        assert_eq!(value, &43);

        let pair = table.get_key_value(&"baz");
        assert_eq!(pair, None);
    }

    #[test]
    fn test_table_capacity() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.capacity(), 2);
        assert_eq!(table.len(), 0);
        assert_eq!(table.is_empty(), true);

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);
        assert_eq!(table.insert(&"baz", 44), false);

        assert_eq!(table.len(), 2);
        assert_eq!(table.get(&"foo"), Some(&42));
        assert_eq!(table.get(&"bar"), Some(&43));
        assert_eq!(table.get(&"baz"), None);
    }

    #[test]
    fn test_table_insert() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);
        assert_eq!(table.insert(&"baz", 44), false);

        assert_eq!(table.len(), 2);
        assert_eq!(table.get(&"foo"), Some(&42));
        assert_eq!(table.get(&"bar"), Some(&43));
        assert_eq!(table.get(&"baz"), None);
    }

    #[test]
    fn test_table_keys() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        let keys = table.keys();

        assert_eq!(keys.len(), 2);
        assert_eq!(&*keys[0].key, "foo");
        assert_eq!(&*keys[1].key, "bar");
    }

    #[test]
    fn test_table_values() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        let values: Vec<&i32> = table.values().iter().collect();
        assert_eq!(values, vec![&42, &43]);
    }

    #[test]
    fn test_table_values_mut() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        for value in table.values_mut() {
            *value += 1;
        }

        let values: Vec<&i32> = table.values().iter().collect();
        assert_eq!(values, vec![&43, &44]);
    }

    #[test]
    fn test_table_iter() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        let items: Vec<(&str, &i32)> = table.iter().collect();
        assert_eq!(items, vec![("foo", &42), ("bar", &43)]);
    }

    #[test]
    fn test_table_iter_mut() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        for (_, value) in table.iter_mut() {
            *value += 1;
        }

        let items: Vec<(&str, &i32)> = table.iter().collect();
        assert_eq!(items, vec![("foo", &43), ("bar", &44)]);
    }

    #[test]
    fn test_table_clear() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        assert_eq!(table.len(), 2);

        table.clear();

        assert_eq!(table.len(), 0);
        assert_eq!(table.is_empty(), true);
    }

    #[test]
    fn test_table_contains_key() {
        let arena = Arena::new(1024);

        let mut table = ArenaTable::<i32>::new(&arena, 2).unwrap();

        assert_eq!(table.insert(&"foo", 42), true);
        assert_eq!(table.insert(&"bar", 43), true);

        assert_eq!(table.contains_key(&"foo"), true);
        assert_eq!(table.contains_key(&"bar"), true);
        assert_eq!(table.contains_key(&"baz"), false);
    }
}
