use core::array;
use std::{
    fmt::Display,
    ops::{Add, Mul},
    usize,
};

///M строки,N столбцы
fn for_each_m_n<F, const M: usize, const N: usize>(mut f: F)
where
    F: FnMut(usize, usize) -> (),
{
    for i in 0..M {
        for j in 0..N {
            f(i, j)
        }
    }
}
///M строки,N столбцы
#[derive(Debug)]
pub struct Matrix<T, const M: usize, const N: usize>([[T; N]; M]);

impl<T, const M: usize, const N: usize> Matrix<T, M, N> {
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize, &T) -> (),
    {
        for_each_m_n::<_, M, N>(|i, j| f(i, j, &self.0[i][j]));
    }
}

impl<T, const M: usize, const N: usize> Matrix<T, M, N>
where
    T: Default,
{
    pub fn map<F, R>(&self, mut f: F) -> Matrix<R, M, N>
    where
        R: Default,
        F: FnMut(usize, usize, &T) -> R,
    {
        let mut zero = Matrix::<R, M, N>::default();
        for_each_m_n::<_, M, N>(|i, j| zero.0[i][j] = f(i, j, &self.0[i][j]));
        zero
    }

    pub fn from_array<const L: usize>(array: [T; L]) -> Matrix<T, M, N>
    where
        T: Clone,
    {
        assert!(M * N == L);
        let mut ret = Self::default();
        //в данной матрице каждая строка M это массив 0..N
        Self::default().map(|i, j, _| ret.0[i][j] = array[j + i * N].clone());

        ret
    }
}

impl<T: 'static, const M: usize, const N: usize> IntoIterator for Matrix<T, M, N> {
    type Item = (usize, usize, T); //row, col, elem

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let ret = self.0.into_iter().enumerate().flat_map(|(row, inner)| {
            inner
                .into_iter()
                .enumerate()
                .map(move |(col, el)| (row, col, el))
        });

        Box::new(ret)
    }
}

impl<T, const M: usize, const N: usize> FromIterator<(usize, usize, T)> for Matrix<T, M, N>
where
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = (usize, usize, T)>>(iter: I) -> Self {
        let mut ret = Matrix::<T, M, N>::default();
        for (row, col, elem) in iter {
            ret.0[row][col] = elem
        }
        ret
    }
}

impl<T, const M: usize, const N: usize> Display for Matrix<T, M, N>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.for_each(|_, col, e| {
            if col == (N - 1) {
                writeln!(f, "{}", e).unwrap();
            } else {
                write!(f, "{}, ", e).unwrap();
            }
        });

        Ok(())
    }
}

impl<T, const M: usize, const N: usize> Default for Matrix<T, M, N>
where
    T: Default,
{
    //default and zero matrix
    fn default() -> Self {
        Matrix(array::from_fn(|_| array::from_fn(|_| T::default())))
    }
}

// Сложение доступно только для матриц одинаковых размеров.
impl<T: 'static, const M: usize, const N: usize> Add for Matrix<T, M, N>
where
    T: Default,
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        //  let mut ret = Self::Output::default();
        self.into_iter()
            .zip(rhs.into_iter())
            .map(|((l_row, l_col, left), (r_row, r_col, right))| {
                assert!(l_row == r_row);
                assert!(l_col == r_col);
                (l_row, l_col, Add::add(left, right))
            })
            .collect()
    }
}

// Умножение доступно для матриц, где количество столбцов первой совпадает с количеством строк второй.
//M строки,N столбцы
impl<T, const M: usize, const N: usize> Matrix<T, M, N>
where
    T: Default + Add<Output = T>,
    for<'a> &'a T: Mul<Output = T>,
{
    fn sum_and_mul_row_col<const P: usize>(
        &self,
        other: &Matrix<T, N, P>,
        row: usize,
        col: usize,
    ) -> T {
        let mut ret = T::default();
        for i in 0..N {
            let mul = &self.0[row][i] * &other.0[i][col];
            ret = ret + mul;
        }
        ret
    }
    //P столбцы другой матрицы
    //при этом N это количество столбцов в self и строк в other
    //результирующая матрица это M x P
    pub fn mul<const P: usize>(self, other: Matrix<T, N, P>) -> Matrix<T, M, P> {
        let mut ret = Matrix::<T, M, P>::default();
        for i in 0..M {
            for j in 0..P {
                ret.0[i][j] = self.sum_and_mul_row_col(&other, i, j)
            }
        }

        ret
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn t() {
        let r = &2 + &3;
        println!("{}", r)
    }

    #[test]
    fn test_from_array() {
        let m1 = Matrix::<i32, 2, 3>::from_array([1, 2, 3, 4, 5, 6]);
        //по индексу от 0,
        // строка 0, колонка 0
        assert!(m1.0[0][0] == 1);
        // строка 1, колонка 2
        assert!(m1.0[1][2] == 6);

        println!("{:?}", m1)
    }

    #[test]
    fn test_into_iter() {
        let initial: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array([1, 2, 3, 4, 5, 6]);
        for (row, col, el) in initial {
            println!("row[{}],col[{}],el[{}]", row, col, el)
        }

        let initial = Matrix::<&str, 2, 3>::from_array(["a", "b", "c", "d", "i", "f"]);
        for (row, col, el) in initial {
            println!("row[{}],col[{}],el[{}]", row, col, el)
        }

        let initial = Matrix::<String, 2, 3>::from_array([
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "I".to_string(),
            "F".to_string(),
        ]);

        let actual: Matrix<String, 2, 3> = initial.into_iter().collect();

        println!("{}", actual);
        assert!(true) //проверяю что этот тест компилится и запускается
    }

    #[test]
    fn test_add_should_work() {
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array([0, 1, 2, 3, 4, 5]);
        let right: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array([0, 1, 2, 3, 4, 5]);

        let actual: Matrix<i32, 2, 3> = left + right;

        for (row, col, elem) in actual {
            print!("elem[{}] ", elem);
            assert_eq!((col + row * 3/*N*/) * 2, elem as usize);
        }
    }

    #[test]
    fn test_mul_should_work() {
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array([0, 1, 2, 3, 4, 5]);
        println!("{}", left);
        let right: Matrix<i32, 3, 2> = Matrix::<i32, 3, 2>::from_array([0, 1, 2, 3, 4, 5]);
        println!("{}", right);

        let actual: Matrix<i32, 2, 2> = left.mul(right);

        println!("{}", actual);

        let left = Matrix::<i32, 2, 3>::from_array([1, 2, 4, 2, 0, 3]);
        println!("{}", left);
        let right = Matrix::<i32, 3, 2>::from_array([2, 5, 1, 3, 1, 1]);
        println!("{}", right);

        let actual: Matrix<i32, 2, 2> = left.mul(right);

        println!("{}", actual);
    }
}
