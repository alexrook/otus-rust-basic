//! Используя типы и трейты из стандартной библиотеки, реализовать:
//! 1. Структуру Cat, хранящий возраст и имя кота. Структура должен уметь:
//!  - Клонироваться.
//!     Рекомендую убедиться, что структуру не получится сделать копируемой.
//!  - Предоставлять отладочную и пользовательскую информацию.
//!  - Преобразовываться в перечисление Pet и обратно.
//!    Перечисление Pet содержит два варианта: Dog и Cat.
//!     Обратите внимание, что преобразование Pet -> Cat может быть невозможно
//!     (вариант Dog).
//!  - Предоставлять ссылку на строковый слайс - имя кота.
//!  - Перегрузить операции сложения и сложения с присваиванием для Cat и целого числа.
//!     Операция должа увеличивать возраст кота.
//! 2. Пример, демонстрирующий все возможности типа Cat.
//! 3. Требования:
//!  - Все перечисленные методы реализованы.
//!  - Все методы, для которых в стандартной библиотеке существует трейт,
//!     реализованы с помощью трейта.
//!  - Пример демонстрирует все возможности типа Cat.
//!  - `cargo clippy` и `cargo fmt --check` не выдают предупреждений и ошибок.

use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone)]
pub struct Cat {
    name: String, //String doesn't implement Copy
    age: u16,
}

impl Cat {
    pub fn new(name: String, age: u16) -> Cat {
        Cat { name, age }
    }
}

impl Display for Cat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A some puffy Cat with name[{}] and age [{}])",
            self.name, self.age
        )
    }
}

impl AsRef<str> for Cat {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

impl TryFrom<Pet> for Cat {
    type Error = &'static str;
    fn try_from(value: Pet) -> Result<Self, Self::Error> {
        if let Pet::Cat(Cat { name, age }) = value {
            Ok(Cat { name, age })
        } else {
            Err("The Pet doesn't contain cat")
        }
    }
}

impl Add<u16> for Cat {
    type Output = Cat;
    fn add(self, rhs: u16) -> Self::Output {
        Cat {
            name: self.name,
            age: self.age + rhs,
        }
    }
}

impl AddAssign<u16> for Cat {
    fn add_assign(&mut self, rhs: u16) {
        self.age += rhs
    }
}

pub enum Pet {
    Cat(Cat),
    Dog { name: String, age: u16 },
}

impl Display for Pet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cat(Cat { name, age }) => {
                write!(f, "A Cat({},{}), a variant of the Pet", name, age)
            }
            Self::Dog { name, age } => write!(f, "A Dog({},{}), a variant of the Pet", name, age),
        }
    }
}

impl From<Cat> for Pet {
    fn from(value: Cat) -> Self {
        Self::Cat(Cat {
            name: value.name,
            age: value.age,
        })
    }
}
