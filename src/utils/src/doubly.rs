use std::mem;
use std::ptr;
use std::cell::Cell;
use std::ops::Index;
use std::ops::IndexMut;
use std::default::Default;
use std::iter::FromIterator;

unsafe fn as_ref<'a, T>(ptr: *const T) -> Option<&'a T> {
    if ptr.is_null() {
        None
    } else {
        Some(&*ptr)
    }
}

unsafe fn as_mut<'a, T>(ptr: *mut T) -> Option<&'a mut T> {
    if ptr.is_null() {
        None
    } else {
        Some(&mut *ptr)
    }
}

pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>
}

pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>
}

struct Node<T> {
    prev:  *mut Node<T>,
    value: T,
    next:  *mut Node<T>
}

pub struct DoublyLinkedList<T> {
    current: Cell<*mut Node<T>>,
    first:   *mut Node<T>,
    last:    *mut Node<T>,
    index:   Cell<isize>,
    length:  usize
}

impl<'a, T> DoublyLinkedList<T> {
    pub fn new() -> DoublyLinkedList<T> {
        DoublyLinkedList {
            current: Cell::new(ptr::null_mut()),
            first:   ptr::null_mut(),
            last:    ptr::null_mut(),
            index:   Cell::new(-1),
            length:  0
        }
    }

    pub fn singleton(v: T) -> DoublyLinkedList<T> {
        unsafe {
            let node = Node::new_on_heap(v);
            DoublyLinkedList {
                current: Cell::new(node),
                first:   node,
                last:    node,
                index:   Cell::new(0),
                length:  1
            }
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn front_mut(&mut self) -> Option<&'a mut T> {
        unsafe { as_mut(self.first).map(|n| { &mut n.value }) }
    }

    pub fn front(&self) -> Option<&'a T> {
        unsafe { as_ref(self.first).map(|n| { &n.value }) }
    }

    pub fn back_mut(&mut self) -> Option<&'a mut T> {
        unsafe { as_mut(self.last).map(|n| { &mut n.value }) }
    }

    pub fn back(&self) -> Option<&'a T> {
        unsafe { as_ref(self.last).map(|n| { &n.value }) }
    }

    pub fn iter(&self) -> Iter<'a, T> {
        unsafe {
            Iter {
                next: if self.first.is_null() {
                    None
                } else {
                    Some(&*self.first)
                }
            }
        }
    }

    pub fn iter_mut(&self) -> IterMut<'a, T> {
        unsafe {
            IterMut {
                next: if self.first.is_null() {
                    None
                } else {
                    Some(&mut *self.first)
                }
            }
        }
    }

    pub fn push_back(&mut self, val: T) {
        unsafe {
            if self.length == 0 {
                *self = DoublyLinkedList::singleton(val);
            } else {
                (*(self.last)).next = Node::new_on_heap(val);
                (*(*(self.last)).next).prev = self.last;
                self.last = (*(self.last)).next;
                self.length += 1;
            }
        }
    }

    pub fn push_front(&mut self, val: T) {
        unsafe {
            if self.length == 0 {
                *self = DoublyLinkedList::singleton(val);
            } else {
                (*(self.first)).prev = Node::new_on_heap(val);
                (*(*(self.first)).prev).next = self.first;
                self.first = (*(self.first)).prev;
                self.length += 1;
                self.index.set(self.index.get() + 1);
            }
        }
    }

    fn index(&self, i: usize) -> Option<&'a T> {
        unsafe {
            if i >= self.length {
                None
            } else {
                self.go_to(i);
                Some(&(*self.current.get()).value)
            }
        }
    }

    fn index_mut(&mut self, i: usize) -> Option<&'a mut T> {
        unsafe {
            if i >= self.length {
                None
            } else  {
                self.go_to(i);
                Some(&mut (*self.current.get()).value)
            }
        }
    }

    fn go_to(&self, i: usize) {
        if self.length / 2 > i {
            if (self.index.get() - i as isize).abs() > i as isize {
                self.go_to_from_start(i);
            } else {
                self.go_to_from_current(i);
            }
        } else if (self.index.get() - i as isize).abs() >= (self.length as isize - i as isize).abs() {
            self.go_to_from_end(i);
        } else {
            self.go_to_from_current(i);
        }
    }

    fn go_to_from_current(&self, i: usize) {
        unsafe {
            while (i as isize) > self.index.get() {
                self.current.set((*self.current.get()).next);
                self.index.set(self.index.get() + 1);
            }
            while (i as isize) < self.index.get() {
                self.current.set((*self.current.get()).prev);
                self.index.set(self.index.get() - 1);
            }
        }
    }

    fn go_to_from_start(&self, i: usize) {
        self.current.set(self.first);
        self.index.set(0);
        self.go_to(i);
    }

    fn go_to_from_end(&self, i: usize) {
        self.current.set(self.last);
        self.index.set(self.length as isize - 1);
        self.go_to(i);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            if self.length == 0 {
                None
            } else {
                let val = ptr::read(&(*self.last).value);
                if self.current.get() == self.last {
                    self.current.set((*self.last).prev);
                    self.index.set(self.index.get() - 1);
                }
                self.length -= 1;
                let old_last = self.last;
                self.last = (*self.last).prev;
                drop(mem::transmute::<_, Box<Node<T>>>(old_last));
                Some(val)
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            if self.length == 0 {
                None
            } else {
                let val = ptr::read(&(*self.first).value);
                if self.current.get() == self.first {
                    self.current.set((*self.first).next);
                }
                self.length -= 1;
                let old_first = self.first;
                self.first = (*self.first).next;
                drop(mem::transmute::<_, Box<Node<T>>>(old_first));
                Some(val)
            }
        }
    }

    pub fn insert(&mut self, i: usize, val: T) {
        if i > self.length {
            panic!("DoublyLinkedList::insert: index out of range");
        } else {
            if self.length == 0 { *self = DoublyLinkedList::singleton(val); return; }
            if i == self.length { self.push_back(val); return; }
            if i == 0           { self.push_front(val); return; }
            unsafe {
                self.go_to(i);
                let new = Node::new_on_heap(val);
                (*new).next = self.current.get();
                (*new).prev = (*self.current.get()).prev;
                (*(*(self.current.get())).prev).next = new;
                (*self.current.get()).prev = new;
                self.current.set(new);
                self.length += 1;
            }
        }
    }

    pub fn remove(&mut self, i: usize) -> T {
        if i >= self.length {
            panic!("DoublyLinkedList::remove: index out of range");
        } else {
            if i == 0 { return self.pop_front().unwrap() }
            if i + 1 == self.length { return self.pop_back().unwrap() }
            unsafe {
                self.go_to(i);

                let val = ptr::read(&(*self.current.get()).value);

                (*(*self.current.get()).next).prev = (*self.current.get()).prev;
                (*(*self.current.get()).prev).next = (*self.current.get()).next;

                let old_node = self.current.get();
                self.current.set((*self.current.get()).next);
                drop(mem::transmute::<_, Box<Node<T>>>(old_node));

                self.length -= 1;

                val
            }
        }
    }

    pub fn append(&mut self, other: &mut DoublyLinkedList<T>) {
        unsafe {
            self.length += other.len();
            (*self.last).next = other.first;
            self.last = other.last;
            *other = Self::new();
        }
    }
}

impl<T> Index<usize> for DoublyLinkedList<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        self.index(i).unwrap()
    }
}

impl<T> IndexMut<usize> for DoublyLinkedList<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        self.index_mut(i).unwrap()
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> DoublyLinkedList<T> {
        DoublyLinkedList::new()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            match self.next {
                None => None,
                Some(node) => {
                    let result = node;
                    self.next = if result.next.is_null() {
                        None
                    } else {
                        Some(&*result.next)
                    };

                    return Some(&(*result).value);
                }
            }
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            match self.next.take() {
                None => None,
                Some(node) => {
                    let result = node;
                    self.next = if result.next.is_null() {
                        None
                    } else {
                        Some(&mut *result.next)
                    };

                    return Some(&mut (*result).value);
                }
            }
        }
    }
}

impl<T> FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<A>(iterator: A) -> Self where A: IntoIterator<Item=T> {
        let mut list = Self::new();

        for x in iterator {
            list.push_back(x);
        }

        return list;
    }
}

impl<T> Node<T> {
    fn new(v: T) -> Node<T> {
        Node {
            prev: ptr::null_mut(),
            value: v,
            next: ptr::null_mut()
        }
    }

    unsafe fn new_on_heap(v: T) -> *mut Node<T> {
        let node = Box::new(Node::new(v));
        let node: *mut Node<T> = mem::transmute(node);
        return node;
    }
}

#[cfg(test)]
mod tests {
    use super::DoublyLinkedList;

    #[test]
    fn test_list() {
        let mut nums = DoublyLinkedList::singleton(5i32);
        assert_eq!(5i32, *nums.back().unwrap());
        assert_eq!(nums.len(), 1);
        nums.push_front(7i32);
        assert_eq!(nums.len(), 2);
        assert_eq!(7i32, *nums.front().unwrap());
        nums.push_front(9i32);
        nums.push_front(6i32);
        assert_eq!(5i32, *nums.index(3).unwrap());
        nums.push_back(10i32);
        assert_eq!(5i32, *nums.index(3).unwrap());
    }

    #[test]
    fn test_len() {
        let mut nums = DoublyLinkedList::singleton(4i32);

        for x in 0..100 { nums.push_front(x); }

        assert_eq!(101, nums.len());

        for x in 0..100 { nums.push_back(x); }

        assert_eq!(201, nums.len());

        nums.insert(40, 8);

        assert_eq!(202, nums.len());

        nums.insert(200, 56);

        assert_eq!(nums.len(), 203);

        for x in 0..24 {
            nums.insert(12, x);
        }

        assert_eq!(nums.len(), 227);
    }

    #[test]
    fn test_iter() {
        let v = vec![16, 29, 42, 1992];

        let list: DoublyLinkedList<i32> = v.iter().cloned().collect();

        assert_eq!(list.iter().cloned().collect::<Vec<i32>>(), v);
    }

    #[test]
    fn test_append() {
        let mut a: DoublyLinkedList<i32> = [12, 14, 18, 29, 40].iter().cloned().collect();
        let mut b: DoublyLinkedList<i32> = [18, 20, 38, 48, 19].iter().cloned().collect();

        let mut v: Vec<i32> = Vec::new();

        for &x in a.iter() {
            v.push(x);
        }

        for &x in b.iter() {
            v.push(x);
        }

        a.append(&mut b);

        assert_eq!(b.len(), 0);

        assert_eq!(a.len(), 10);

        assert_eq!(v, a.iter().cloned().collect::<Vec<i32>>());
    }

    #[test]
    fn test_empty() {
        let mut dl = DoublyLinkedList::new();

        assert!(dl.is_empty());

        dl.push_front("string");

        assert!(!dl.is_empty());
    }
}