//! - Возвращать итератор по всем элементам,
//!  -  добавлять элемент в конец,
//!  -  добавлять элемент в начало,
//!  - добавлять элемент после N-го,
//!  - Разделяться на два списка: от начального элемента до (N-1)-го и от (N-1)-го до последнего.
//!  - Предоставлять возможность изменять элементы списка.

pub enum LinkedList<T> {
    Nil,
    Cons { head: T, tail: Box<LinkedList<T>> },
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self::Nil
    }

    pub fn from_iter_rev(iter: impl Iterator<Item = T>) -> Self {
        let ret = Self::new();
        iter.fold(ret, |acc, elem| acc.prepend(elem))
    }

    //O(1)
    pub fn prepend(self, elem: T) -> Self {
        LinkedList::Cons {
            head: elem,
            tail: Box::new(self),
        }
    }
    //O(n)
    pub fn append(self, elem: T) -> Self {
        match self {
            LinkedList::Nil => Self::new().prepend(elem),
            LinkedList::Cons { head, tail } => {
                let tail = *tail;
                LinkedList::Cons {
                    head,
                    tail: Box::new(tail.append(elem)),
                }
            }
        }
    }

    //O(n)
    pub fn insert(self, idx: usize, elem: T) -> Self {
        let mut opt: Option<T> = Some(elem);
        self.into_iter()
            .enumerate()
            .fold(Self::new(), |acc, (i, e)| {
                let mut next = acc.append(e);
                if idx == i {
                    next = next.append(opt.take().unwrap())
                }
                next
            })
    }

    pub fn replace(self, idx: usize, elem: T) -> Self {
        let mut opt: Option<T> = Some(elem);
        self.into_iter()
            .enumerate()
            .fold(Self::new(), |acc, (i, e)| {
                if idx == i {
                    acc.append(opt.take().unwrap())
                } else {
                    acc.append(e)
                }
            })
    }

    pub fn pop(&self) -> (Option<&T>, &Self) {
        match self {
            LinkedList::Cons { head, tail } => (Some(head), tail),
            LinkedList::Nil => (None, &LinkedList::Nil),
        }
    }

    pub fn head(&self) -> Option<&T> {
        match self {
            LinkedList::Cons { head, tail: _ } => Some(head),
            LinkedList::Nil => None,
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter(Some(self))
    }

    pub fn split(self, n: usize) -> (LinkedList<T>, LinkedList<T>) {
        let zero = (Self::new(), Self::new());
        self.into_iter()
            .enumerate()
            .fold(zero, |(left, right), (idx, elem)| {
                if idx < n {
                    (left.append(elem), right)
                } else {
                    (left, right.append(elem))
                }
            })
    }
}

pub struct Iter<'a, T>(Option<&'a LinkedList<T>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.take() {
            Some(LinkedList::Cons { head, tail }) => {
                self.0 = Some(&tail); //deref ?
                Some(head)
            }
            _ => None,
        }
    }
}

pub struct IntoIter<T>(Option<LinkedList<T>>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.take() {
            Some(LinkedList::Cons { head, tail }) => {
                self.0 = Some(*tail);
                Some(head)
            }
            _ => None,
        }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Some(self))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_creation() {
        let vec = vec![1, 2, 3];
        println!("{:?}", vec.iter().next());
        vec.iter().for_each(|x| println!("{x}"));
        let list = LinkedList::new();
        let list = list.prepend(1);
        let list = list.prepend(42);
        let list = list.prepend(55);
        let (h1, ll2) = list.pop();

        assert_eq!(h1, Some(&55));
        assert_eq!(ll2.head(), Some(&42));

        let list2 = LinkedList::new().prepend(1).prepend(2).prepend(3);

        assert_eq!(list2.head(), Some(&3));
    }

    #[test]
    fn test_append() {
        let list = LinkedList::new().prepend(1).prepend(2).prepend(3);

        let list = list.append(21).append(42);

        for (idx, elem) in list.into_iter().enumerate() {
            let expected = match idx {
                0 => 3,
                1 => 2,
                2 => 1,
                3 => 21,
                4 => 42,
                _ => panic!("Something wrong with your code"),
            };

            assert_eq!(elem, expected);
        }
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
            .for_each(|(x, y)| assert_eq!(x, y));

        second
            .into_iter()
            .zip([2, 1])
            .for_each(|(x, y)| assert_eq!(x, y));
    }

    #[test]
    fn test_iter() {
        let list = LinkedList::new();
        let list = list.prepend(1);
        let list = list.prepend(42);
        let list = list.prepend(55);

        let iter = list.iter();

        assert_eq!(iter.count(), 3);

        for (i, e) in list.iter().enumerate() {
            let m = match i {
                0 => 55,
                1 => 42,
                2 => 1,
                _ => panic!("something wrong this your code"),
            };

            assert_eq!(e, &m);
        }
    }

    #[test]
    fn test_into_iter() {
        let list = LinkedList::new().prepend(1).prepend(2).prepend(3);
        let mut idx = 4;
        for elem in list.into_iter() {
            idx -= 1;
            assert_eq!(idx, elem);
            println!("{elem}");
        }
    }

    #[test]
    fn test_from_iter() {
        let vec = vec![1, 2, 3, 42];
        let list = LinkedList::from_iter_rev(vec.into_iter());

        for x in list {
            println!("{x}");
        }
    }
}
