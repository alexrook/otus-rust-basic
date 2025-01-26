use std::{
    iter::Product,
    marker::PhantomData,
    ops::{Add, Deref, Index, Mul},
};

use crate::matrix::Matrix;

/// Время 'b и PhantomData остались от попытки реализовать Index, такой,
/// что бы время жизни ссылки по индексу, превышало время жизни структуры.
/// При реализации Deref (thanks Discord people), 'b не нужно
pub struct Matrices<'a, 'b, T, const M: usize, const N: usize>(
    Vec<&'a Matrix<T, M, N>>,
    PhantomData<&'b T>,
);

//Эта реализация Index компилируюся, но не работает как ожидалось
//see test_index_should_work
// impl<'a, 'b, T, const M: usize, const N: usize> Index<usize> for Matrices<'a, 'b, T, M, N> {
//     type Output = Matrix<T, M, N>;

//     fn index(&self, index: usize) -> &'a Self::Output {
//         self.0.index(index)
//     }
// }

impl<'a, 'b, T, const M: usize, const N: usize> Deref for Matrices<'a, 'b, T, M, N> {
    type Target = Vec<&'a Matrix<T, M, N>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    'a: 'b,
{
    pub fn new(vec: Vec<&'a Matrix<T, M, N>>) -> Matrices<'a, 'b, T, M, N> {
        Matrices(vec, PhantomData)
    }

    //не удалось реализовать через трайт Index -> error[E0597]: actual does not live long enough
    pub fn my_index(&'b self, index: usize) -> &'a Matrix<T, M, N> {
        //это метод рабочий
        self.0[index]
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    T: Default + 'static,
    for<'c> &'c T: Add<Output = T>,
{
    pub fn sum_el(self) -> T {
        let mut ret = T::default();
        for matrix in self.0 {
            for el in matrix.into_iter() {
                ret = &ret + el;
            }
        }
        ret
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    T: Default + 'static,
    for<'c> &'c T: Mul<Output = T>,
{
    pub fn mul_el(self) -> T {
        let mut prev: &T = &T::default();
        let mut buf = T::default();
        let mut first_time = true;
        for matrix in self.0 {
            for el in matrix.into_iter() {
                if first_time {
                    prev = el;
                    first_time = false;
                } else {
                    buf = prev * el;
                    prev = &buf;
                }
            }
        }

        buf
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    T: 'static + Product<&'a T>,
{
    pub fn product_el(self) -> T {
        let r = self.0.into_iter().flat_map(|matrix| matrix.into_iter());
        r.product()
    }
}

#[cfg(test)]
pub mod tests {

    use super::Matrices;
    use crate::matrices::*;

    #[test]
    fn test_my_index_should_work() {
        let array = [0, 1, 2, 3, 4, 5];

        let m1: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let m2: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let should_be_m1: &Matrix<i32, 2, 3>;
        let should_be_m2: &Matrix<i32, 2, 3>;
        {
            let actual = Matrices::new(vec![&m1, &m2]);
            should_be_m1 = actual.my_index(0);
            should_be_m2 = actual.my_index(1);
        }
        assert_eq!(should_be_m1, &m1);
        assert_eq!(should_be_m2, &m2)
    }

    #[ignore]
    #[test]
    fn test_index_should_work() {
        //fail
        let array = [0, 1, 2, 3, 4, 5];

        let m1 = Matrix::<i32, 2, 3>::from_array(&array);
        let m2 = Matrix::<i32, 2, 3>::from_array(&array);
        //  let should_be_m1;

        {
            let actual = Matrices::new(vec![&m1, &m2]);
            //  should_be_m1 = actual.index(0); //error here
        }
        //assert_eq!(should_be_m1, &m1);
    }

    #[test]
    fn test_deref_should_work() {
        let array = [0, 1, 2, 3, 4, 5];

        let m1 = Matrix::<i32, 2, 3>::from_array(&array);
        let m2 = Matrix::<i32, 2, 3>::from_array(&array);
        let should_be_m1;
        let should_be_m2;

        {
            let actual = Matrices::new(vec![&m1, &m2]);
            should_be_m1 = actual[0];
            should_be_m2 = actual[1];
        }
        assert_eq!(should_be_m1, &m1);
        assert_eq!(should_be_m2, &m2);
    }

    #[test]
    fn test_sum_el_should_work() {
        let array = [0, 1, 2, 3, 4, 5];
        let should_be_sum: i32 = 2 * array.iter().sum::<i32>();
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let right: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);

        let actual = Matrices::new(vec![&left, &right]);

        let actual_sum = actual.sum_el();

        assert_eq!(actual_sum, should_be_sum)
    }

    #[test]
    fn test_mul_el_should_work() {
        let array = [1, 2, 3, 4, 5, 6];
        let should_be_mul: i32 = array.iter().fold(1, |acc, el| acc * *el * *el);
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let right: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let actual = Matrices::new(vec![&left, &right]);
        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul);

        let should_be_mul: i32 = 1 * 4 * 3 * 4 * 5 * 6 * //m1
                                 3 * 4 * 3 * 7 * 12 * 6;
        let m1: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[1, 4, 3, 4, 5, 6]);
        let m2: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let actual = Matrices::new(vec![&m1, &m2]);
        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul);

        let should_be_mul: i32 = 0; //bcs m1 has zero
        let m1: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 4, 3, 4, 5, 6]);
        let m2: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let m3: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let actual = Matrices::new(vec![&m1, &m2, &m3]);
        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul)
    }

    #[test]
    fn test_product_should_work() {
        let array = [1, 2, 3, 4, 5, 6];
        let should_be_mul: i32 = array.iter().fold(1, |acc, el| acc * *el * *el);
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let right: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let actual = Matrices::new(vec![&left, &right]);
        let actual_mul = actual.product_el();
        assert_eq!(actual_mul, should_be_mul);

        let should_be_mul: i32 = 1 * 4 * 3 * 4 * 5 * 6 * //m1
                                 3 * 4 * 3 * 7 * 12 * 6;
        let m1: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[1, 4, 3, 4, 5, 6]);
        let m2: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let actual = Matrices::new(vec![&m1, &m2]);
        let actual_mul = actual.product_el();
        assert_eq!(actual_mul, should_be_mul);

        let should_be_mul: i32 = 0; //bcs m1 has zero
        let m1: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 4, 3, 4, 5, 6]);
        let m2: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let m3: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[3, 4, 3, 7, 12, 6]);
        let actual = Matrices::new(vec![&m1, &m2, &m3]);
        let actual_mul = actual.product_el();
        assert_eq!(actual_mul, should_be_mul)
    }
}
