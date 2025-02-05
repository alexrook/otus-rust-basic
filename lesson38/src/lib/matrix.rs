use core::array;
use std::{
    fmt::Display,
    ops::{Add, Index, Mul},
};

pub fn for_each_m_n<F, const M: usize, const N: usize>(mut f: F)
where
    F: FnMut(usize, usize),
{
    for row/*строки*/ in 0..M {
        for col/*столбцы*/ in 0..N {
            f(row, col)
        }
    }
}

pub fn is_first_element(row: usize, col: usize) -> bool {
    row == 0 && col == 0
}

pub fn non_first_element(row: usize, col: usize) -> bool {
    !is_first_element(row, col)
}
///M строки,N столбцы
#[derive(Debug, PartialEq, Eq)]
pub struct Matrix<T, const M: usize, const N: usize>([[T; N]; M]);

//к сожалению derive на работает для Default для [[T; N]; M]
impl<T, const M: usize, const N: usize> Default for Matrix<T, M, N>
where
    T: Default,
{
    //default and zero matrix
    fn default() -> Self {
        Matrix(array::from_fn(|_| array::from_fn(|_| T::default())))
    }
}

impl<T, const M: usize, const N: usize> Index<(usize, usize)> for Matrix<T, M, N> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.0[row][col]
    }
}

impl<T, const M: usize, const N: usize> Matrix<T, M, N> {
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(MatrixIterEntry<&T>),
    {
        for_each_m_n::<_, M, N>(|row, col| {
            f(MatrixIterEntry {
                row,
                col,
                elem: &self.0[row][col],
            })
        });
    }

    pub fn fold<F>(&self, mut f: F) -> T
    where
        F: FnMut(Option<&T>, MatrixIterEntry<&T>) -> T,
    {
        const {
            if M * N == 0 {
                panic!("M * N == 0");
            }
        }

        let mut acc: T = f(
            None,
            MatrixIterEntry {
                row: 0,
                col: 0,
                elem: &self.0[0][0],
            },
        );

        self.for_each(|entry| {
            if non_first_element(entry.row, entry.col) {
                //skip "used" elem
                acc = f(Some(&acc), entry);
            }
        });

        acc
    }

    pub fn fold_mul(&self) -> T
    where
        T: Clone, //для операции сложения на ссылке нужен или identity elem или Clone
        for<'c> &'c T: Mul<Output = T>,
    {
        self.fold(|may_be_acc, entry| match may_be_acc {
            Some(acc) => acc * entry.elem,
            None => entry.elem.clone(),
        })
    }

    pub fn fold_sum(&self) -> T
    where
        T: Clone, //для операции умножения на ссылке нужен или identity elem или Clone
        for<'a> &'a T: Add<Output = T>,
    {
        self.fold(|may_be_acc, entry| match may_be_acc {
            Some(acc) => acc + entry.elem,
            None => entry.elem.clone(),
        })
    }

    pub fn from_array<const L: usize>(array: &[T; L]) -> Matrix<T, M, N>
    where
        T: Clone,
    {
        const {
            if L != M * N {
                panic!("L != M * N");
            }
        }
        let first_elem = array[0].clone(); //клонирование первого элемента во все ячейки позволяет избавиться от Default
        let mut ret = Matrix(array::from_fn(|_| array::from_fn(|_| first_elem.clone())));
        //в данной матрице каждая строка M это массив 0..N
        for_each_m_n::<_, M, N>(|row, col| ret.0[row][col] = array[col + row * N].clone());

        ret
    }

    pub fn mul_scalar(self, v: T) -> Self
    where
        T: Default + Clone + 'static,
        T: Mul<Output = T>,
    {
        let mut ret = Matrix::<T, M, N>::default();
        for entry in self {
            ret.0[entry.row][entry.col] = entry.elem * v.clone()
        }
        ret
    }
}

//Структура элемент итератора для матрицы
pub struct MatrixIterEntry<T> {
    pub row: usize,
    pub col: usize,
    pub elem: T,
}

impl<T: 'static, const M: usize, const N: usize> IntoIterator for Matrix<T, M, N> {
    type Item = MatrixIterEntry<T>;

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let ret = self.0.into_iter().enumerate().flat_map(|(row, inner)| {
            inner
                .into_iter()
                .enumerate()
                .map(move |(col, elem)| MatrixIterEntry { row, col, elem })
        });

        Box::new(ret)
    }
}

impl<'a, T: 'static, const M: usize, const N: usize> IntoIterator for &'a Matrix<T, M, N> {
    type Item = MatrixIterEntry<&'a T>;

    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let ret = self.0.iter().enumerate().flat_map(|(row, inner)| {
            inner
                .into_iter()
                .enumerate()
                .map(move |(col, elem)| MatrixIterEntry { row, col, elem })
        });

        Box::new(ret)
    }
}

impl<T, const M: usize, const N: usize> FromIterator<MatrixIterEntry<T>> for Matrix<T, M, N>
where
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = MatrixIterEntry<T>>>(iter: I) -> Self {
        let mut ret = Matrix::<T, M, N>::default();
        for entry in iter {
            ret.0[entry.row][entry.col] = entry.elem
        }
        ret
    }
}

impl<T, const M: usize, const N: usize> Display for Matrix<T, M, N>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.for_each(|entry| {
            if entry.col == (N - 1) {
                writeln!(f, "{}", entry.elem).unwrap();
            } else {
                write!(f, "{}, ", entry.elem).unwrap();
            }
        });

        Ok(())
    }
}

// Сложение доступно только для матриц одинаковых размеров.
impl<T: 'static, const M: usize, const N: usize> Add for Matrix<T, M, N>
where
    T: Default, //see FromIterator
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        //  let mut ret = Self::Output::default();
        self.into_iter()
            .zip(rhs)
            .map(|(left, right)| {
                assert!(left.row == right.row);
                assert!(left.col == right.col);
                MatrixIterEntry {
                    row: left.row,
                    col: left.col,
                    elem: Add::add(left.elem, right.elem),
                }
            })
            .collect()
    }
}

// Умножение доступно для матриц, где количество столбцов первой совпадает с количеством строк второй.
// M,N - строки и столбцы self
// N,P - cтроки и столбцы rhs
impl<T, const M: usize, const N: usize, const P: usize> Mul<Matrix<T, N, P>> for Matrix<T, M, N>
where
    T: Default + Add<Output = T>,
    for<'a> &'a T: Mul<Output = T>,
{
    type Output = Matrix<T, M, P>;

    fn mul(self, rhs: Matrix<T, N, P>) -> Self::Output {
        let mut ret = Matrix::<T, M, P>::default();
        for row in 0..M {
            for col in 0..P {
                //умножение self.строки и rhs.столбца
                let mut mul_sum_row_col = T::default();
                for i in 0..N {
                    let mul = &self.0[row][i] * &rhs.0[i][col];
                    mul_sum_row_col = mul_sum_row_col + mul;
                }
                //и присвоение результата колонка*столбец элементу новой матрицы
                ret.0[row][col] = mul_sum_row_col
            }
        }

        ret
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let matrix = Matrix::<i32, 2, 3>::default();
        for entry in matrix {
            assert_eq!(entry.elem, i32::default())
        }
    }

    #[test]
    fn test_from_array() {
        let m1 = Matrix::<i32, 2, 3>::from_array(&[1, 2, 3, 4, 5, 6]);
        //по индексу от 0,
        // строка 0, колонка 0
        assert!(m1.0[0][0] == 1);
        // строка 1, колонка 2
        assert!(m1.0[1][2] == 6);
        println!("{:?}", m1);
    }

    #[test]
    fn test_fold() {
        let array = [1, 2, 3, 4, 5, 6];
        let m1 = Matrix::<i32, 2, 3>::from_array(&array);
        let actual = m1.fold(|prev, entry| match prev {
            Some(acc) => acc + entry.elem,
            None => entry.elem + 0, /*identity elem for add */
        });

        assert_eq!(actual, array.iter().sum());

        let array = [1, 2, 3, 4, 5, 6];
        let m1 = Matrix::<i32, 2, 3>::from_array(&array);
        let actual = m1.fold(|prev, entry| match prev {
            Some(acc) => acc * entry.elem,
            None => entry.elem * 1, /*identity elem for mul */
        });

        assert_eq!(actual, array.iter().product());
    }

    #[test]
    fn test_into_iter() {
        let initial: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[1, 2, 3, 4, 5, 6]);
        for MatrixIterEntry { row, col, elem } in initial {
            println!("row[{}],col[{}],el[{}]", row, col, elem)
        }

        let initial = Matrix::<&str, 2, 3>::from_array(&["a", "b", "c", "d", "i", "f"]);
        for MatrixIterEntry { row, col, elem } in initial {
            println!("row[{}],col[{}],el[{}]", row, col, elem)
        }

        let initial = Matrix::<String, 2, 3>::from_array(&[
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
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 1, 2, 3, 4, 5]);
        let right: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 1, 2, 3, 4, 5]);

        let actual: Matrix<i32, 2, 3> = left + right;

        for MatrixIterEntry { row, col, elem } in actual {
            print!("elem[{}] ", elem);
            assert_eq!((col + row * 3/*N*/) * 2, elem as usize);
        }
    }

    #[test]
    fn test_mul_should_work() {
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 1, 2, 3, 4, 5]);
        let right: Matrix<i32, 3, 2> = Matrix::<i32, 3, 2>::from_array(&[0, 1, 2, 3, 4, 5]);
        let actual: Matrix<i32, 2, 2> = left.mul(right);
        println!("{}", actual);
        assert_eq!(actual, Matrix::<i32, 2, 2>::from_array(&[10, 13, 28, 40]));

        let left = Matrix::<i32, 2, 3>::from_array(&[1, 2, 4, 2, 0, 3]);
        let right = Matrix::<i32, 3, 2>::from_array(&[2, 5, 1, 3, 1, 1]);
        let actual: Matrix<i32, 2, 2> = left.mul(right);
        println!("{}", actual);
        assert_eq!(actual, Matrix::<i32, 2, 2>::from_array(&[8, 15, 7, 13]));
    }

    #[test]
    fn test_mul_scalar_should_work() {
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[0, 1, 2, 3, 4, 5]);
        let actual = left.mul_scalar(42);
        println!("{}", actual);
        assert_eq!(
            actual,
            Matrix::<i32, 2, 3>::from_array(&[0, 42, 84, 126, 168, 210])
        );

        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[5, 4, 3, 2, 1, 0]);
        let actual = left.mul_scalar(-2);
        println!("{}", actual);
        assert_eq!(
            actual,
            Matrix::<i32, 2, 3>::from_array(&[-10, -8, -6, -4, -2, 0])
        );

        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&[5, 4, 3, 2, 1, 0]);
        let actual = left.mul_scalar(0);
        println!("{}", actual);
        assert_eq!(actual, Matrix::<i32, 2, 3>::default());

        let array = [5, 4, 3, 2, 1, 0];
        let left: Matrix<i32, 2, 3> = Matrix::<i32, 2, 3>::from_array(&array);
        let actual = left.mul_scalar(1);
        println!("{}", actual);
        assert_eq!(actual, Matrix::<i32, 2, 3>::from_array(&array));
    }
}
