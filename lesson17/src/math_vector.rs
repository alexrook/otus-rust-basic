// Реализовать обобщённый тип Vector. (Hint: использовать массив i32 соответствующего размера для хранения данных)
// Реализовать метод-конструктор Vector::from(arr), который принимает массив, и возврщает тип Vector
// Реализовать конструкторы Vector::new1(x), Vector::new2(x, y), Vector::new3(x, y, z) для векторов размером 1, 2 и 3 соответственно
// Реализовать метод len(), который возвращает длину (модуль) вектора. sqrt(x*x + y*y)
// Обобщить тип Vector для числового типа. (Hint: создать типаж Num)
// Реализовать сложение векторов с помощью трейта std::ops::Add. (x1, y1) + (x2, y2) = (x1+x2, y1+y2)
// Реализовать умножение вектора на скаляр с помощью std::ops::Mul. (x, y) * n = (x * n, y * n)

use std::ops::{Add, Mul};

pub trait Num: Add<Output = Self> + Mul<Output = Self> + Default + Sized + Clone {}

impl Num for u32 {}

pub struct Vector<E: Num, const N: usize> {
    pub holder: [E; N],
}

impl<E: Num + Copy, const N: usize> Vector<E, N> {
    pub fn from(array: [E; N]) -> Vector<E, N> {
        Vector { holder: array }
    }

    pub fn new1(x: E) -> Vector<E, 1> {
        Vector::from([x])
    }

    pub fn new2(x: E, y: E) -> Vector<E, 2> {
        Vector::from([x, y])
    }

    pub fn new3(x: E, y: E, z: E) -> Vector<E, 3> {
        Vector::from([x, y, z])
    }

    pub fn len(&self) -> E {
        let mut acc: E = E::default();
        for x in self.holder.iter() {
            acc = acc + *x * *x
        }
        acc
    }

    //Реализовать сложение векторов с помощью трейта std::ops::Add. (x1, y1) + (x2, y2) = (x1+x2, y1+y2)
    //TODO: remove Copy use std::array::from_fn
    pub fn sum(&mut self, b: Vector<E, N>) -> Vector<E, N> {
        let mut ret = Self::from([E::default(); N]);
        for (i, e) in self.holder.iter().enumerate() {
            ret.holder[i] = *e + b.holder[i]
        }
        ret
    }

    // Реализовать умножение вектора на скаляр с помощью std::ops::Mul. (x, y) * n = (x * n, y * n)
    pub fn scalar_mul(&self, b: E) -> Vector<E, N> {
        let mut ret = Self::from([E::default(); N]);
        for (i, e) in self.holder.iter().enumerate() {
            ret.holder[i] = *e * b
        }
        ret
    }
}

//TODO: Vec Add, Mul
