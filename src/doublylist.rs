use core::{
    marker::PhantomData,
    ptr::NonNull,
    iter::FromIterator,
};

type Link<T> = Option<NonNull<Node<T>>>;

trait IntoNonNull<T> {
    fn into_non_null(self) -> NonNull<T>;
}

impl<T> IntoNonNull<T> for Box<T> {
    #[inline(always)]
    fn into_non_null(self) -> NonNull<T> {
        // We know a box is always nonnull
        unsafe { NonNull::new_unchecked(Box::into_raw(self)) }
    }
}

struct Node<T> {
    item: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    #[inline(always)]
    pub const fn new(item: T) -> Self {
        Self {
            item,
            next: None,
            prev: None,
        }
    }

    #[inline(always)]
    pub const fn with_next(item: T, next: Link<T>) -> Self {
        Self {
            item,
            next,
            prev: None,
        }
    }

    #[inline(always)]
    pub const fn with_prev(item: T, prev: Link<T>) -> Self {
        Self {
            item,
            prev,
            next: None,
        }
    }

    #[inline(always)]
    pub fn into_item(self: Box<Self>) -> T {
        self.item
    }
}

/// A doubly list.
pub struct DoublyList<T> {
    head: Link<T>,
    tail: Link<T>,
    marker: PhantomData<Box<Node<T>>>,
}

impl <T: Clone> Clone for DoublyList<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}
impl<T> DoublyList<T> {
    /*
     * Pushing to the front:
     *
     * Create a new boxed node on the heap,
     * that has its next pointer set to
     * the current head.
     *
     * Then convert the boxed node to a raw pointer.
     *
     * Set the current head's previous pointer the
     * newly created node.
     *
     * set the current head to the newly created node.
     *
     * if the tail is none (means the list was empty),
     * set the tail to newly created node.
     *
     * Imagine the following list, and we try to push 0:
     *
     * INITIAL LIST
     * =============================================
     * None <- 1 <-> 2 <-> 3 <-> 4 -> None,
     *         |                 |
     *         |                 |
     *        HEAD              TAIL
     * =============================================
     *
     * 1)   We create a new node, with its next element
     *      set to the head of the list, and its previous
     *      element to None
     * =============================================
     *  None <- 1 <-> 2 <-> 3 <-> 4 -> None
     *         /|                 |
     *        / |                 |
     *        | HEAD             TAIL
     *        \
     * None <- 0
     * =============================================
     *
     * 2)   Here we set the heads previous element
     *      to the new element.
     * =============================================
     *  None <- 0 <-> 1 <-> 2 <-> 3 <-> 4 -> None
     *                |                 |
     *                |                 |
     *               HEAD              TAIL
     * =============================================
     *
     * 3) We move the head to the new element
     * =============================================
     * None <- 0 <-> 1 <-> 2 <-> 3 <-> 4 -> None
     *         |                       |
     *         |                       |
     *        HEAD                    TAIL
     * =============================================
     *
     */
    #[inline(always)]
    fn push_front_node(&mut self, mut node: Box<Node<T>>) {
        unsafe {
            node.next = self.head;
            node.prev = None;

            let node = Some(Box::into_non_null(node));

            match self.head {
                None => self.tail = node,
                Some(head) => (*head.as_ptr()).prev = node,
            }

            self.head = node;
        }
    }

    #[inline(always)]
    fn push_back_node(&mut self, mut node: Box<Node<T>>) {
        unsafe {
            node.next = None;
            node.prev = self.tail;

            let node = Some(Box::into_non_null(node));

            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).next = node,
            }

            self.tail = node;
        }
    }

    /*
     * 1). We start of with this:
     *======================
     * 0 <-> 1 <-> 2 <-> 3
     * ^                 ^
     * |                 |
     * |                 |
     * head             tail
     *======================
     *
     * 2). The we move the head one,
     * =====================
     * 0 <-> 1 <-> 2 <-> 3
     *      /           |
     *     /            |
     *    /             |
     *  head           tail
     *
     * 3). Lastly we must set the prv
     * of the new head to None,
     * =======================
     *  0  1 <-> 2 <-> 3
     *     |           |
     *     |           |
     *     |           |
     *    head        tail
     * ======================
     */
    #[inline(always)]
    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                Some(head) => (*head.as_ptr()).prev = None,
            }

            node
        })
    }

    #[inline(always)]
    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        self.tail.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.tail = node.prev;

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).next = None,
            }

            node
        })
    }
}

impl<T> DoublyList<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            marker: PhantomData,
        }
    }
    
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    #[inline(always)]
    pub fn push_front(&mut self, item: T) {
        self.push_front_node(Box::new(Node::new(item)));
    }

    #[inline(always)]
    pub fn push_back(&mut self, item: T) {
        self.push_back_node(Box::new(Node::new(item)));
    }

    #[inline(always)]
    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_item)
    }

    #[inline(always)]
    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(Node::into_item)
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            head: self.head,
            tail: self.tail,
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            marker: PhantomData,
        }
    }
}

impl<T> Drop for DoublyList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

impl <'a, T> IntoIterator for &'a DoublyList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl <'a, T> IntoIterator for &'a mut DoublyList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl <T> IntoIterator for DoublyList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl <T> FromIterator<T> for DoublyList<T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>
    {
        let mut list = DoublyList::new();

        for item in iter {
            list.push_back(item)
        }

        list
    }
}

pub struct Iter<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Clone for Iter<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Iter { ..*self }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.head, self.tail) {
            (Some(head), Some(tail)) => {
                let node = unsafe { &*head.as_ptr() };
                if head.as_ptr() as usize == tail.as_ptr() as usize {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.head = node.next;
                }

                Some(&node.item)
            }

            _ => None,
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        match (self.head, self.tail) {
            (Some(head), Some(tail)) => {
                let node = unsafe { &*tail.as_ptr() };

                if head.as_ptr() as usize == tail.as_ptr() as usize {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.tail = node.prev;
                }

                Some(&node.item)
            }

            _ => None,
        }
    }
}

pub struct IterMut<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.head, self.tail) {
            (Some(head), Some(tail)) => {
                let node = unsafe { &mut *head.as_ptr() };
                if head.as_ptr() as usize == tail.as_ptr() as usize {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.head = node.next;
                }

                Some(&mut node.item)
            }

            _ => None,
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        match (self.head, self.tail) {
            (Some(head), Some(tail)) => {
                let node = unsafe { &mut *tail.as_ptr() };

                if head.as_ptr() as usize == tail.as_ptr() as usize {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.tail = node.prev;
                }

                Some(&mut node.item)
            }

            _ => None,
        }
    }
}

#[repr(transparent)]
pub struct IntoIter<T> {
    inner: DoublyList<T>,
}

impl <T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

impl <T> DoubleEndedIterator for IntoIter<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut list = DoublyList::new();

        list.push_front(0);
        list.push_front(10);
        list.push_back(20);
        list.push_front(11);
        list.push_back(21);
        // list = 11 10 20 21
        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&11));
        assert_eq!(iter.next_back(), Some(&21));
        assert_eq!(iter.next(), Some(&10));
        assert_eq!(iter.next_back(), Some(&20));
        assert_eq!(iter.next_back(), Some(&0));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
