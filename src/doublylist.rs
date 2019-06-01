use core::{
    fmt::{self, Debug},
    iter::{ExactSizeIterator, FromIterator, FusedIterator},
    marker::PhantomData,
    ptr::NonNull,
};

type Link<T> = Option<NonNull<Node<T>>>;

trait IntoNonNull {
    type Item: ?Sized;
    fn into_non_null(self) -> NonNull<Self::Item>;
}

impl<T: ?Sized> IntoNonNull for Box<T> {
    type Item = T;

    #[inline(always)]
    fn into_non_null(self) -> NonNull<Self::Item> {
        // We know a box is always nonnull
        unsafe { NonNull::new_unchecked(Box::into_raw(self)) }
    }
}

pub struct Node<T: ?Sized, U: ?Sized = T> {
    next: Link<U>,
    prev: Link<U>,
    item: T,
}

impl<T, U: ?Sized> Node<T, U> {
    #[inline(always)]
    pub const fn new(item: T) -> Self {
        Self {
            item,
            next: None,
            prev: None,
        }
    }

    #[inline(always)]
    pub fn boxed(item: T) -> Box<Self> {
        Box::new(Self::new(item))
    }
}

impl<T> Node<T> {
    #[inline(always)]
    pub fn into_item(self: Box<Self>) -> T {
        self.item
    }
}

/// A doubly list.
pub struct DoublyList<T: ?Sized> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>,
}

unsafe impl<T: ?Sized + Send> Send for DoublyList<T> {}
unsafe impl<T: ?Sized + Sync> Sync for DoublyList<T> {}

impl<T: ?Sized + Clone> Clone for DoublyList<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: ?Sized + Debug> Debug for DoublyList<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: ?Sized> DoublyList<T> {
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
    fn push_front_node_private(&mut self, mut node: Box<Node<T>>) {
        unsafe {
            node.next = self.head;
            node.prev = None;

            let node = Some(Box::into_non_null(node));

            match self.head {
                None => self.tail = node,
                Some(head) => (*head.as_ptr()).prev = node,
            }

            self.head = node;
            self.len += 1;
        }
    }

    #[inline(always)]
    fn push_back_node_private(&mut self, mut node: Box<Node<T>>) {
        unsafe {
            node.next = None;
            node.prev = self.tail;

            let node = Some(Box::into_non_null(node));

            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).next = node,
            }

            self.tail = node;
            self.len += 1;
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
    fn pop_front_node_private(&mut self) -> Option<Box<Node<T>>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                Some(head) => (*head.as_ptr()).prev = None,
            }

            self.len -= 1;
            node
        })
    }

    #[inline(always)]
    fn pop_back_node_private(&mut self) -> Option<Box<Node<T>>> {
        self.tail.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.tail = node.prev;

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).next = None,
            }

            self.len -= 1;
            node
        })
    }
}

impl<T: ?Sized> DoublyList<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    #[inline(always)]
    pub fn peek_front(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.head.as_ref().map(|node| &node.as_ref().item)
            }
        }
    }

    #[inline(always)]
    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.head.as_mut().map(|node| &mut node.as_mut().item)
            }
        }
    }

    #[inline(always)]
    pub fn peek_back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.tail.as_ref().map(|node| &node.as_ref().item)
            }
        }
    }

    #[inline(always)]
    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.tail.as_mut().map(|node| &mut node.as_mut().item)
            }
        }
    }

    #[inline(always)]
    pub fn push_front_node(&mut self, node: Box<Node<T>>) {
        self.push_front_node_private(node)
    }

    #[inline(always)]
    pub fn push_back_node(&mut self, node: Box<Node<T>>) {
        self.push_back_node_private(node)
    }

    #[inline(always)]
    pub fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        self.pop_front_node_private()
    }

    #[inline(always)]
    pub fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        self.pop_back_node_private()
    }

    #[inline(always)]
    pub const fn iter(&self) -> Iter<T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }
}

impl<T> DoublyList<T> {
    #[inline(always)]
    pub fn push_front(&mut self, item: T) {
        self.push_front_node(Node::boxed(item));
    }

    #[inline(always)]
    pub fn push_back(&mut self, item: T) {
        self.push_back_node(Node::boxed(item));
    }

    #[inline(always)]
    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_item)
    }

    #[inline(always)]
    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(Node::into_item)
    }
}

impl<T: ?Sized> Drop for DoublyList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front_node() {}
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a DoublyList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a mut DoublyList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for DoublyList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl<T> FromIterator<T> for DoublyList<T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut list = DoublyList::new();

        for item in iter {
            list.push_back(item)
        }

        list
    }
}

pub struct Iter<'a, T: ?Sized> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

unsafe impl<T: ?Sized + Send> Send for Iter<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for Iter<'_, T> {}

impl<T: ?Sized> Copy for Iter<'_, T> {}

impl<T: ?Sized> Clone for Iter<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Iter { ..*self }
    }
}

impl<T: ?Sized + Debug> Debug for Iter<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.len).finish()
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &*node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &node.item
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: ?Sized> DoubleEndedIterator for Iter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &*node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &node.item
            })
        }
    }
}

impl<T: ?Sized> FusedIterator for Iter<'_, T> {}
impl<T: ?Sized> ExactSizeIterator for Iter<'_, T> {}

pub struct IterMut<'a, T: ?Sized> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    marker: PhantomData<&'a mut Node<T>>,
}

unsafe impl<T: ?Sized + Send> Send for IterMut<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for IterMut<'_, T> {}

impl<T: ?Sized + Debug> Debug for IterMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IterMut").field(&self.len).finish()
    }
}

impl<'a, T: ?Sized> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &mut node.item
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: ?Sized> DoubleEndedIterator for IterMut<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.tail.map(|node| unsafe {
                // Need an unbound lifetime to get 'a
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.tail = node.prev;
                &mut node.item
            })
        }
    }
}

impl<T: ?Sized> FusedIterator for IterMut<'_, T> {}
impl<T: ?Sized> ExactSizeIterator for IterMut<'_, T> {}

pub struct IntoIter<T> {
    inner: DoublyList<T>,
}

impl<T: Debug> Debug for IntoIter<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.inner).finish()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_back()
    }
}

impl<T> FusedIterator for IntoIter<T> {}
impl<T> ExactSizeIterator for IntoIter<T> {}

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

    #[test]
    fn test_unsized_list() {
        use core::any::Any;

        let mut list = DoublyList::new();

        let node: Box<Node<dyn Any>> = Node::boxed(10usize);

        list.push_front_node(node);
    }
}
