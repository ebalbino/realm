use super::Arena;

/// A singly-linked list that is allocated in an arena. Each push operation
/// appends a new element to the end of the list by allocating a new node in the
/// arena.
#[derive(Debug, PartialEq)]
pub struct List<T> {
    arena: *const Arena,
    count: usize,
    generation: usize,
    head: Option<*mut Node<T>>,
    tail: Option<*mut Node<T>>,
}

/// A node in a singly-linked list.
#[derive(Debug, PartialEq)]
pub struct Node<T> {
    next: Option<*mut Node<T>>,
    value: T,
}


impl<T> List<T> {
    pub fn new(arena: &Arena) -> Self {
        let generation = arena.generation();

        List {
            arena,
            count: 0,
            head: None,
            tail: None,
            generation,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn push(&mut self, value: T) -> Option<()> {
        let arena = unsafe { &*self.arena };
        let mut node = arena.push(Node {
            next: None,
            value,
        })?;

        let node_ptr = Some(node.as_mut_ptr());

        match self.tail {
            None => {
                self.head = node_ptr;
                self.tail = node_ptr;
                self.count = 1;
            }
            Some(ptr) => unsafe {
                let tail = &mut *ptr;
                tail.next = node_ptr;
                self.tail = node_ptr;
                self.count += 1;
            },
        }

        Some(())
    }

    pub fn pop(&mut self) -> Option<&T> {
        match self.head {
            None => None,
            Some(ptr) => {
                let node = unsafe { &*ptr };
                self.head = node.next;
                self.count -= 1;

                if self.head.is_none() {
                    self.tail = None;
                }

                Some(&node.value)
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let mut current = self.head;

        std::iter::from_fn(move || {
            match current {
                None => None,
                Some(ptr) => unsafe {
                    let node = &*ptr;
                    current = node.next;
                    Some(&node.value)
                },
            }
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let mut current = self.head;

        std::iter::from_fn(move || {
            match current {
                None => None,
                Some(ptr) => unsafe {
                    let node = &mut *ptr;
                    current = node.next;
                    Some(&mut node.value)
                },
            }
        })
    }

    pub fn last(&self) -> Option<&T> {
        match self.tail {
            None => None,
            Some(ptr) => unsafe { Some(&(*ptr).value) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let arena = Arena::new(1024);
        let mut list = List::new(&arena);

        list.push(42);
        list.push(43);
        list.push(44);

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&42));
        assert_eq!(iter.next(), Some(&43));
        assert_eq!(iter.next(), Some(&44));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_list_mut() {
        let arena = Arena::new(1024);
        let mut list = List::new(&arena);

        list.push(42);
        list.push(43);
        list.push(44);

        let mut iter = list.iter_mut();

        assert_eq!(iter.next(), Some(&mut 42));
        assert_eq!(iter.next(), Some(&mut 43));
        assert_eq!(iter.next(), Some(&mut 44));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_list_pop() {
        let arena = Arena::new(1024);
        let mut list = List::new(&arena);

        list.push(42);
        list.push(43);
        list.push(44);

        assert_eq!(list.pop(), Some(&42));
        assert_eq!(list.pop(), Some(&43));
        assert_eq!(list.pop(), Some(&44));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_list_len() {
        let arena = Arena::new(1024);
        let mut list = List::new(&arena);

        assert_eq!(list.len(), 0);

        list.push(42);
        list.push(43);
        list.push(44);

        assert_eq!(list.len(), 3);

        list.pop();
        list.pop();

        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_list_empty() {
        let arena = Arena::new(1024);
        let mut list = List::new(&arena);

        assert_eq!(list.count, 0);
        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);

        list.push(42);
        list.push(43);
        list.push(44);

        assert_eq!(list.count, 3);
        assert_ne!(list.head, None);
        assert_ne!(list.tail, None);
        assert_eq!(list.last(), Some(&44));

        list.pop();
        list.pop();
        list.pop();

        assert_eq!(list.count, 0);
        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);
        assert_eq!(list.last(), None);
    }
}
