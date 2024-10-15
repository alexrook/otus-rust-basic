//! Односвязный список
//!Цель:
//!
//! В рамках этого ДЗ реализуем односвязный список.
//!
//! Описание/Пошаговая инструкция выполнения домашнего задания:
//!
//! Каждый элемент такого списка содержит объект и указатель на следующий элемент. Таким образом, элементы списка объединины в цепь, в которой каждый элемент знает о следующем.
//! Подробности на вики: https://clck.ru/332iN9
//! Список должен уметь:
//!  - Возвращать итератор по всем элементам,
//!  -  добавлять элемент в конец,
//!  -  добавлять элемент в начало,
//!  - добавлять элемент после N-го,
//!  - Разделяться на два списка: от начального элемента до (N-1)-го и от (N-1)-го до последнего.
//!  - Предоставлять возможность изменять элементы списка.
//!  - Так как каждый элемент списка содержит ссылку на следующий,
//!     Rust не даст нам менять элементы списка
//!         (правило заимствования о одной мутабельной ссылке).
//!         Для преодоления этого ограничения можно использовать обёртку Rc<RefCell>.
//!         Она даст возможность модифицировать элемент списка несмотря на то,
//!          что на него существует ссылка (у предыдущего элемента).
//!
//! Требования:
//! 1. Все перечисленные методы реализованы.
//! 2. Все методы протестированы.
//! 3. Написан пример кода, демонстрирующий функционал списка.
//! 4. `cargo clippy`` и `cargo fmt --check`` не выдают предупреждений и ошибок.

use std::rc::Rc;

pub trait LinkedList<T> {
    fn as_cons(&mut self) -> Option<&mut Cons<T>>;
}

// struct LinkedListIter<T>(Option<Rc<Cons<T>>>);
// impl<T> Iterator for LinkedListIter<T> {
//     type Item = Rc<T>;
//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(rcLL) = &self.0 {
//             if let Some(cons) = Rc::into_inner(Rc::clone(rcLL)) {
//                 let Cons{head, tail} = cons;
//                  if let Some(innerCons) = tail.as_cons()  {
//                      self.0 =Some(Rc::new(*innerCons) ); //ERROR
//                  } else {
//                     self.0 = None
//                  }
//                 Some(head)
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }
// struct LinkedListIter<T>(Rc<dyn LinkedList<T>>);

// impl<T> Iterator for LinkedListIter<T> {

// }

// impl<T> Iterator for dyn LinkedList<T> {
//     type Item = Rc<T>;
//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(cons) = self.as_cons() {
//             let ret = cons.head.clone();
//             if let Some(tail) =Rc::get_mut(&mut cons.tail){
//                 *self = tail
//             } else {
//                 self = &mut Nil
//             }
//             Some(ret)
//         } else {
//             None
//         }
//     }
// }
pub struct Cons<T> {
    pub head: Rc<T>,
    pub tail: Rc<dyn LinkedList<T>>,
}

impl<T> LinkedList<T> for Cons<T> {
    fn as_cons(&mut self) -> Option<&mut Cons<T>> {
        Some(self)
    }
}

struct Nil;

impl<T> LinkedList<T> for Nil {
    fn as_cons(&mut self) -> Option<&mut Cons<T>> {
        None
    }
}

impl<T> dyn LinkedList<T> {
    pub fn empty() -> impl LinkedList<T> {
        Nil
    }

    pub fn one(v: T) -> impl LinkedList<T> {
        Cons {
            head: Rc::new(v),
            tail: Rc::new(Nil),
        }
    }

    pub fn new(h: T, t: Rc<dyn LinkedList<T>>) -> impl LinkedList<T> {
        Cons {
            head: Rc::new(h),
            tail: t,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_creation() {
        let _ = <dyn LinkedList<u32>>::empty();
        let one = <dyn LinkedList<u32>>::one(42);
        let one_str = Cons {
            head: Rc::new("AAA".to_string()),
            tail: Rc::new(Nil),
        };

        let _ = <dyn LinkedList<u32>>::new(154, Rc::new(one));
        let _ = <dyn LinkedList<String>>::new("BBB".to_string(), Rc::new(one_str));
    }
}
