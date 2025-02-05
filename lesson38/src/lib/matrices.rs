use std::ops::{Add, Deref, Mul};

use crate::matrix::Matrix;

pub struct Matrices<'a, 'b, T, const M: usize, const N: usize>(Vec<&'a Matrix<&'b T, M, N>>);

impl<'a, 'b, T, const M: usize, const N: usize> Deref for Matrices<'a, 'b, T, M, N> {
    type Target = Vec<&'a Matrix<&'b T, M, N>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    'a: 'b,
{
    pub fn new(vec: Vec<&'a Matrix<&'a T, M, N>>) -> Matrices<'a, 'b, T, M, N> {
        Matrices(vec)
    }

    pub fn my_index<'c>(&'c self, index: usize) -> &'a Matrix<&'b T, M, N> {
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
    'a: 'b,
    T: Default + 'static,
    for<'c> &'c T: Add<Output = T>,
{
    pub fn sum_el(self) -> T {
        let mut ret = T::default();
        for matrix in self.0 {
            matrix.for_each(|entry| ret = &ret + entry.elem);
        }
        ret
    }
}

impl<'a, 'b, T, const M: usize, const N: usize> Matrices<'a, 'b, T, M, N>
where
    T: Clone + 'static,
    for<'c> &'c T: Mul<Output = T>,
{
    pub fn mul_el(self) -> T {
        let mut buf = self[0][(0, 0)].clone(); //самый первый элемент самой первой матрицы
        let mut non_first_time = false;

        for matrix in self.0 {
            for row in 0..M {
                for col in 0..N {
                    if non_first_time {
                        buf = &buf * matrix[(row, col)]
                    } else {
                        non_first_time = true; //самый первый элемент пропускаем
                    }
                }
            }
        }

        buf
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_my_index_should_work() {
        let array = [&0, &1, &2, &3, &4, &5];

        let m1: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let m2: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let should_be_m1: &Matrix<&i32, 2, 3>;
        let should_be_m2: &Matrix<&i32, 2, 3>;
        {
            let actual = Matrices::new(vec![&m1, &m2]);
            should_be_m1 = actual.my_index(0);
            should_be_m2 = actual.my_index(1);
        }
        assert_eq!(should_be_m1, &m1);
        assert_eq!(should_be_m2, &m2)
    }

    #[test]
    fn test_deref_should_work() {
        let array = [&0, &1, &2, &3, &4, &5];

        let m1: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let m2: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let should_be_m1: &Matrix<&i32, 2, 3>;
        let should_be_m2: &Matrix<&i32, 2, 3>;

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
        let array = [&0, &1, &2, &3, &4, &5];
        let should_be_sum: i32 = 2 * (0 + 1 + 2 + 3 + 4 + 5);
        let left: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let right: Matrix<&i32, 2, 3> = Matrix::from_array(&array);

        let actual = Matrices::new(vec![&left, &right]);
        let actual_sum = actual.sum_el();
        assert_eq!(actual_sum, should_be_sum)
    }

    #[test]
    fn test_mul_el_should_work() {
        let array = [&1, &2, &3, &4, &5, &6];
        let should_be_mul: i32 = array.iter().fold(1, |acc, el| acc * *el * *el);
        eprintln!("{}", should_be_mul);
        let left: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let right: Matrix<&i32, 2, 3> = Matrix::from_array(&array);
        let actual = Matrices::new(vec![&left, &right]);

        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul);

        let array1 = [&1, &4, &3, &4, &5, &6];
        let array2 = [&3, &4, &3, &7, &12, &6];
        let should_be_mul: i32 = 1 * 4 * 3 * 4 * 5 * 6 * //m1
                                 3 * 4 * 3 * 7 * 12 * 6;
        let m1: Matrix<&i32, 2, 3> = Matrix::<&i32, 2, 3>::from_array(&array1);
        let m2: Matrix<&i32, 2, 3> = Matrix::from_array(&array2);
        let actual = Matrices::new(vec![&m1, &m2]);
        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul);

        let should_be_mul: i32 = 0; //bcs m1 has zero
        let m1: Matrix<&i32, 2, 3> = Matrix::from_array(&[&0, &4, &3, &4, &5, &6]);
        let m2: Matrix<&i32, 2, 3> = Matrix::from_array(&[&3, &4, &3, &7, &12, &6]);
        let m3: Matrix<&i32, 2, 3> = Matrix::from_array(&[&3, &4, &3, &7, &12, &6]);
        let actual = Matrices::new(vec![&m1, &m2, &m3]);

        let actual_mul = actual.mul_el();
        assert_eq!(actual_mul, should_be_mul)
    }
}
