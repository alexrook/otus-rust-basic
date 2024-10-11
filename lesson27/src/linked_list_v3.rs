use std::rc::Rc;
#[derive(Debug)]
pub enum LinkedList<T> {
    Cons {
        head: Rc<T>,
        tail: Rc<LinkedList<T>>,
    },
    Nil,
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList::Nil
    }

    //O(1)
    pub fn prepend(self: Self, v: T) -> LinkedList<T> {
        self.prepend_rc(Rc::new(v))
    }

    fn prepend_rc(self: Self, v: Rc<T>) -> LinkedList<T> {
        LinkedList::Cons {
            head: v,
            tail: Rc::new(self),
        }
    }

    //O(n)
    pub fn append(self, v: T) -> LinkedList<T> {
        self.append_rc(Rc::new(v))
    }

    pub fn append_rc(self, v: Rc<T>) -> LinkedList<T> {
        Self::append_loop(Rc::new(self), v)
    }

    //O(n^2) ?
    pub fn insert(self, n: usize, v: T) -> LinkedList<T> {
        let rc_v = Rc::new(v);
        if n == 0 {
            self.prepend_rc(rc_v)
        } else {
            self.into_iter()
                .enumerate()
                .fold(Self::new(), |acc, (idx, elem)| {
                    if idx == n {
                        acc.append_rc(elem).append_rc(rc_v.clone())
                    } else {
                        acc.append_rc(elem)
                    }
                })
        }
    }

    //O(n^2) ?
    pub fn replace(self, n: usize, v: T) -> LinkedList<T> {
        let rc_v = Rc::new(v);
        self.into_iter()
            .enumerate()
            .fold(Self::new(), |acc, (idx, elem)| {
                if idx == n {
                    acc.append_rc(rc_v.clone())
                } else {
                    acc.append_rc(elem)
                }
            })
    }

    //O(n^2)
    pub fn split(self, n: usize) -> (LinkedList<T>, LinkedList<T>) {
        let zero = (Self::new(), Self::new());
        self.into_iter()
            .enumerate()
            .fold(zero, |(left, right), (idx, el)| {
                if idx < n {
                    (left.append_rc(el), right)
                } else {
                    (left, right.append_rc(el))
                }
            })
    }

    //private
    fn append_loop(xa: Rc<LinkedList<T>>, v: Rc<T>) -> LinkedList<T> {
        match &*xa {
            LinkedList::Cons { head, tail } => {
                let new_tail = LinkedList::append_loop(Rc::clone(tail), v);
                LinkedList::Cons {
                    head: Rc::clone(head),
                    tail: Rc::new(new_tail),
                }
            }
            LinkedList::Nil => LinkedList::Cons {
                head: v,
                tail: Rc::new(LinkedList::Nil),
            },
        }
    }
}

pub struct IntoIter<T>(Rc<LinkedList<T>>);

impl<T> Iterator for IntoIter<T> {
    type Item = Rc<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ret, tail) = match &*self.0 {
            LinkedList::Nil => return None,
            LinkedList::Cons { head, tail } => (Some(Rc::clone(head)), tail),
        };
        self.0 = Rc::clone(tail);
        ret
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = Rc<T>;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Rc::new(self))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_prepend() {
        let list = LinkedList::new().prepend(1).prepend(2).prepend(3);

        for (idx, x) in list.into_iter().enumerate() {
            assert_eq!(idx + *x, 3) //3 + 0 == 2 + 1 == 1 + 2
        }
    }

    #[test]
    fn test_append() {
        let list = LinkedList::new().append(1).append(2).append(3);

        for (idx, x) in list.into_iter().enumerate() {
            assert_eq!(idx + 1, *x)
        }
    }

    #[test]
    fn test_insert() {
        let list = LinkedList::new().append(1).append(2).append(3);

        let list = list.insert(1, 4);

        list.into_iter()
            .zip(vec![1, 2, 4, 3])
            .for_each(|(x, y)| assert_eq!(x.as_ref(), &y));
    }

    #[test]
    fn test_replace() {
        let list = LinkedList::new().append(1).append(2).append(3);
        //in the middle
        let list = list.replace(1, 42);

        list.into_iter()
            .zip(vec![1, 42, 3])
            .for_each(|(x, y)| assert_eq!(x.as_ref(), &y));

        //at first
        let list = LinkedList::new().append(1).append(2).append(3);
        let list = list.replace(0, 42);

        list.into_iter()
            .zip(vec![42, 2, 3])
            .for_each(|(x, y)| assert_eq!(x.as_ref(), &y));
    }

    #[test]
    fn test_split() {
        let list = LinkedList::new()
            .prepend(1)
            .prepend(2)
            .prepend(3)
            .prepend(42);

        let (first, second) = list.split(2);

        first
            .into_iter()
            .zip([42, 3])
            .for_each(|(x, y)| assert_eq!(*x, y));

        second
            .into_iter()
            .zip([2, 1])
            .for_each(|(x, y)| assert_eq!(*x, y));
    }
}
