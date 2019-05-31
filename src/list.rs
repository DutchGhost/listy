use core::iter::FromIterator;

/// The type used to link to another Node.
///
/// Due to NonNull optimizations,
/// the size of this structure
/// is the same as the size of a regular pointer
type Link<T> = Option<Box<Node<T>>>;

/// A node holds a value, and a pointer to a next node.
#[derive(Debug)]
struct Node<T> {
    /// The value this node holds,
    item: T,

    /// The next element of the list
    next: Link<T>,
}

impl<T> Node<T> {
    /// Returns a new node, with it next element set to `None`.
    #[inline(always)]
    pub const fn new(item: T) -> Self {
        Self { item, next: None }
    }

    /// Returns a new node, with it next element set to `next`.
    #[inline(always)]
    pub const fn new_with_next(item: T, next: Link<T>) -> Self {
        Self { item, next }
    }
}

/// A list of nodes.
#[derive(Debug)]
#[repr(transparent)]
pub struct List<T> {
    /// Hold just the head of the list
    head: Link<T>,
}

impl<T> List<T> {
    /// Returns a new empty list.
    /// # Examples
    /// ```
    /// # use lists::list::List;
    /// const list: List<i32> = List::new();
    ///
    /// assert!(list.is_empty());
    /// ```
    #[inline(always)]
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Returns `true` if the list is empty, false otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Pushes a new item to the front of the list.
    /// # Examples
    /// ```
    /// # use lists::list::List;
    /// let mut list = List::new();
    /// list.push(10);
    /// list.push(20);
    ///
    /// // The list contains 2 items,
    /// // we get them out by popping again.
    ///
    /// assert_eq!(list.pop(), Some(20));
    /// assert_eq!(list.pop(), Some(10));
    /// ```
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        let new_node = Box::new(Node::new_with_next(item, self.head.take()));
        self.head = Some(new_node);
    }

    /// Pops the last pushed item from the list.
    /// # Examples
    /// ```
    /// # use lists::list::List;
    /// let mut list = (0..3).collect::<List<_>>();
    ///
    /// assert_eq!(list.pop(), Some(2));
    /// assert_eq!(list.pop(), Some(1));
    /// assert_eq!(list.pop(), Some(0));
    /// assert_eq!(list.pop(), None);
    /// ```
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.item
        })
    }

    /// Returns a reference to the head of the list.
    /// # Examples
    /// ```
    /// # use lists::list::List;
    /// let mut list = List::new();
    /// list.push(10);
    ///
    /// assert_eq!(list.peek(), Some(&10));
    /// ```
    #[inline(always)]
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|ref node| &node.item)
    }

    /// Returns a mutable reference to the head of the list.
    /// # Examples
    /// ```
    /// # use lists::list::List;
    /// let mut list = List::new();
    /// list.push(10);
    ///
    /// list.peek_mut().map(|elem| {
    ///     *elem += 20;
    /// });
    ///
    /// assert_eq!(list.pop(), Some(30));
    /// ```
    #[inline(always)]
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.item)
    }

    /// Returns an iterator over the list,
    /// that yields references to the elements in the list.
    #[inline(always)]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.head.as_ref().map(|node| &**node),
        }
    }

    /// Returns iterator over the list,
    /// that yields mutable references to the elements in the list.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            inner: self.head.as_mut().map(|node| &mut **node),
        }
    }

    /// Splits the list at the first element after the closure.
    pub fn split_after<F>(&mut self, mut splitter: F) -> Option<List<T>>
    where
        F: FnMut(&T) -> bool,
    {
        let mut iter = self.iter_mut();

        loop {
            match iter.peek() {
                Some(item) => {
                    if splitter(item) {
                        break;
                    }
                }
                None => break,
            }

            match iter.next() {
                Some(_) => continue,
                None => break,
            }
        }

        let IterMut { mut inner } = iter;

        match inner.take() {
            Some(node) => Some(List {
                head: node.next.take(),
            }),
            None => None,
        }
    }
}

// An iterative drop,
// because the default drop behaviour is recursive!
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cursor = self.head.take();

        while let Some(mut node) = cursor {
            cursor = node.next.take();
        }
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl<T> FromIterator<T> for List<T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut list = List::new();

        for item in iter {
            list.push(item)
        }

        list
    }
}

/// An iterator over a list of nodes.
#[repr(transparent)]
pub struct Iter<'a, T> {
    inner: Option<&'a Node<T>>,
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter { ..*self }
    }
}

impl<'a, T> Iter<'a, T> {
    #[inline(always)]
    fn peek(&self) -> Option<&T> {
        self.inner.as_ref().map(|node| &node.item)
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take().map(|ref node| {
            self.inner = node.next.as_ref().map(|node| &**node);
            &node.item
        })
    }
}

/// A mutable iterator over a list of nodes.
#[repr(transparent)]
pub struct IterMut<'a, T> {
    inner: Option<&'a mut Node<T>>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline(always)]
    fn peek(&self) -> Option<&T> {
        self.inner.as_ref().map(|node| &node.item)
    }

    #[inline(always)]
    fn peek_mut(&mut self) -> Option<&mut T> {
        self.inner.as_mut().map(|node| &mut node.item)
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take().map(|node| {
            self.inner = node.next.as_mut().map(|node| &mut **node);
            &mut node.item
        })
    }
}

/// An iterator over owned items in the list.
#[repr(transparent)]
pub struct IntoIter<T> {
    inner: List<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        // 4 3 2 1
        let mut list = (0..5).collect::<List<u32>>();

        let mut splitted = list.split_after(|x| *x == 3).unwrap();

        assert_eq!(splitted.pop(), Some(2));
    }
}
