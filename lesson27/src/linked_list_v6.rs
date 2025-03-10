use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(Debug)]
pub struct LinkedList<T> {
    pub head: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

type Splitted<T> = (
    Box<dyn Iterator<Item = Rc<RefCell<Node<T>>>>>,
    Box<dyn Iterator<Item = Rc<RefCell<Node<T>>>>>,
);

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub next: Link<T>,
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList { head: None }
    }

    pub fn one(v: T) -> LinkedList<T> {
        Self::new().prepend_a(v)
    }

    pub fn head(&self) -> Option<&RefCell<Node<T>>> {
        self.head.as_ref().map(|inner| inner.as_ref())
    }

    pub fn head_value(&self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| {
            // Заимствуем весь узел
            let borrowed: Ref<'_, Node<T>> = node.borrow();
            Ref::map(borrowed, |n| &n.value)
        })
    }

    pub fn prepend_a(self, v: T) -> LinkedList<T> {
        match self.head.as_ref() {
            Some(inner) => {
                let new_node: Rc<RefCell<Node<T>>> = Self::new_node(v);
                new_node.borrow_mut().next = Some(Rc::clone(inner));
                LinkedList {
                    head: Some(new_node),
                }
            }

            None => LinkedList {
                head: Some(Self::new_node(v)),
            },
        }
    }

    pub fn prepend_b(&mut self, v: T) -> &mut Self {
        let new_head = Rc::new(RefCell::new(Node {
            value: v,
            next: None,
        }));
        self.head.take().into_iter().for_each(|inner| {
            new_head.borrow_mut().next = Some(inner);
        });

        self.head = Some(new_head);
        self
    }

    pub fn append(&mut self, v: T) -> &mut Self {
        match self.iter().last().take() {
            Some(inner) => inner.borrow_mut().next = Some(Self::new_node(v)),
            None => {
                self.head = Some(Self::new_node(v));
            }
        }

        self
    }

    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.clone()) //это должно увеличить счетчик Rc без клонирования T
    }

    pub fn replace(&mut self, i: usize, v: T) -> Option<T> {
        let mut tmp: Option<T> = Some(v);
        let mut ret: Option<T> = None;
        self.iter().enumerate().for_each(|(idx, inner)| {
            if idx == i {
                let old_node = inner.replace_with(|old| Node {
                    value: tmp.take().unwrap(),
                    next: old.next.clone(),
                });
                ret = Some(old_node.value);
            }
        });
        ret
    }

    pub fn insert(&mut self, i: usize, v: T) {
        let mut tmp: Option<T> = Some(v);
        self.iter().enumerate().for_each(|(idx, inner)| {
            if idx == i {
                let mut m = inner.borrow_mut();
                m.next = Some(Rc::new(RefCell::new(Node {
                    value: tmp.take().unwrap(),
                    next: m.next.clone(),
                })))
            }
        });
    }

    pub fn split(&mut self, i: usize) -> Splitted<T>
    where
        T: 'static,
    {
        let left = self
            .iter()
            .enumerate()
            .take_while(move |(idx, _)| *idx < i)
            .map(|(_, el)| el);

        let right = self
            .iter()
            .enumerate()
            .skip_while(move |(idx, _)| *idx < i)
            .map(|(_, el)| el);

        (Box::new(left), Box::new(right))
    }

    //private
    fn new_node(v: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Self::new_node_a(v)))
    }

    fn new_node_a(v: T) -> Node<T> {
        Node {
            value: v,
            next: None,
        }
    }
}

pub struct Iter<T>(Link<T>);

impl<T> Iterator for Iter<T> {
    type Item = Rc<RefCell<Node<T>>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().inspect(|inner| {
            self.0 = inner.borrow().next.clone(); //это должно увеличить счетчик Rc без клонирования T
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_prepend() {
        let list = LinkedList::new().prepend_a(1).prepend_a(2).prepend_a(3);
        list.iter()
            .zip([3, 2, 1])
            .for_each(|(inner, x)| assert_eq!(inner.borrow().value, x));

        let mut list = LinkedList::new();
        list.prepend_b(1).prepend_b(2).prepend_b(42);

        list.iter()
            .zip([42, 2, 1])
            .for_each(|(inner, x)| assert_eq!(inner.borrow().value, x));
    }

    #[test]
    fn test_append() {
        let mut list = LinkedList::new();
        list.append(1);
        assert_eq!(
            list.head()
                .map(|v: &RefCell<Node<i32>>| { *Ref::map(v.borrow(), |node| &node.value) }),
            Some(1)
        );
        assert_eq!(
            list.iter()
                .last()
                .map(|v| { *Ref::map(v.borrow(), |node| &node.value) }),
            Some(1)
        );
        list.append(2);
        assert_eq!(
            list.head()
                .map(|v: &RefCell<Node<i32>>| { *Ref::map(v.borrow(), |node| &node.value) }),
            Some(1)
        );
        assert_eq!(
            list.iter()
                .last()
                .map(|v| { *Ref::map(v.borrow(), |node| &node.value) }),
            Some(2)
        );
        list.append(21).append(42);
        list.iter()
            .zip([1, 2, 21, 42])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));

        list.iter()
            .zip([1, 2, 21, 42])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));

        let mut iter = list.iter();
        assert_eq!(iter.next().map(|v| v.borrow().value), Some(1));
        assert_eq!(iter.next().map(|v| v.borrow().value), Some(2));
        assert_eq!(iter.next().map(|v| v.borrow().value), Some(21));
    }

    #[test]
    fn test_replace_1() {
        let mut list = LinkedList::new();
        list.append(1).append(2).append(3);
        list.replace(1, 42);
        list.iter()
            .zip([1, 42, 3])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
        list.replace(2, 84);
        list.iter()
            .zip([1, 42, 84])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
    }

    #[test]
    fn test_replace_2() {
        let mut list = LinkedList::new();
        list.append("1").append("2").append("3");
        list.replace(1, "42");
        list.iter()
            .zip(["1", "42", "3"])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
        list.replace(2, "84");
        list.iter()
            .zip(["1", "42", "84"])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
    }

    #[test]
    fn test_insert() {
        let mut list = LinkedList::new();
        list.append("1").append("2").append("3");
        list.insert(1, "42");
        list.iter()
            .zip(["1", "2", "42", "3"])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
        list.insert(3, "84");
        list.iter()
            .zip(["1", "2", "42", "3", "84"])
            .for_each(|(elem, x)| assert_eq!(elem.borrow().value, x));
    }

    #[test]
    fn test_split() {
        let mut list = LinkedList::new();
        list.append("1").append("2").append("3").append("42");
        let (left, right) = list.split(2);

        left.zip(["1", "2"])
            .for_each(|(x, y)| assert_eq!(x.borrow().value, y));

        right
            .zip(["3", "42"])
            .for_each(|(x, y)| assert_eq!(x.borrow().value, y));
    }
}
