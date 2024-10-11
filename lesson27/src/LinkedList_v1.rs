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
//! Так как каждый элемент списка содержит ссылку на следующий,
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

pub trait LinkedList<T>: IntoIterator {}

struct Cons<T> {
    head: Option<Rc<T>>,
    tail: Option<Rc<Cons<T>>>,
}

struct ConsIter<T>(Option<Rc<Cons<T>>>);

impl<T> Iterator for ConsIter<T> {
    type Item = Rc<T>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.clone() {
            None => None,
            Some(rc) => {
                self.0 = rc.tail.clone();
                rc.head.clone()
            }
        }
    }
}

impl<T> IntoIterator for Cons<T> {
    type Item = Rc<T>;
    type IntoIter = ConsIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        ConsIter(Some(self.into()))
    }
}

impl<T> LinkedList<T> for Cons<T> {}

