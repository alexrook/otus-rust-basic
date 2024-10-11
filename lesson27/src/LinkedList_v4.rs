use std::rc::Rc;
#[derive(Debug)]
pub enum LinkedList<T> {
    Cons {
        head: Rc<T>,
        tail: Rc<LinkedList<T>>,
    },
    Nil,
}

impl<T> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => LinkedList::Nil,
            Self::Cons { head, tail } => Self::Cons {
                head: Rc::clone(head),
                tail: Rc::clone(tail),
            },
        }
    }
}

struct Iter<T>(Option<Rc<LinkedList<T>>>);

impl<T> Iterator for Iter<T> {
    type Item = Rc<T>;
    fn next(&mut self) -> Option<Self::Item> {
        // self.0.take().map(|inner| {
        //    let cloned = Rc::clone(&inner);
           
        //     cloned.to_owned()
           
        // });
        self.0 = None;
        None
    }
}
