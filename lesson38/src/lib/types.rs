use std::ops::{Add, Mul};

#[derive(Debug)]
pub enum Error {
    InvalidLen,
}

pub struct MatrixCol<'c, 'm, const N: usize, T> {
    matrixes: &'c [&'m Matrix<N, T>],
}

#[derive(Debug, PartialEq)]
pub struct Matrix<const N: usize, T> {
    elements: Vec<T>,
}

impl<const N: usize, T> Matrix<N, T> {
    pub fn from_vec(vec: Vec<T>) -> Result<Self, Error> {
        if vec.len() % N != 0 {
            Err(Error::InvalidLen)
        } else {
            Ok(Self { elements: vec })
        }
    }
}

impl<const N: usize, T> Matrix<N, T>
where
    T: Add<Output = T> + Clone,
{
    pub fn sum(&self) -> Option<T> {
        let mut sum: Option<T> = None;
        for (i, el) in self.elements.iter().enumerate() {
            if i == 0 {
                sum = Some(el.clone())
            } else {
                sum = Some(sum.unwrap() + el.clone())
            }
        }
        sum
    }
}

impl<const N: usize, T> Matrix<N, T>
where
    T: Mul<Output = T> + Clone,
{
    pub fn prod(&self) -> Option<T> {
        let mut prod: Option<T> = None;
        for (i, el) in self.elements.iter().enumerate() {
            if i == 0 {
                prod = Some(el.clone())
            } else {
                prod = Some(prod.unwrap() * el.clone())
            }
        }
        prod
    }
}

impl<const N: usize, T> Add<T> for Matrix<N, T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        Self {
            elements: self
                .elements
                .into_iter()
                .map(|x| x + other.clone())
                .collect::<Vec<T>>(),
        }
    }
}

impl<const N: usize, T> Mul<T> for Matrix<N, T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;
    fn mul(self, other: T) -> Self::Output {
        Self {
            elements: self
                .elements
                .into_iter()
                .map(|x| x * other)
                .collect::<Vec<T>>(),
        }
    }
}

impl<'s, 'm, const N: usize, T> MatrixCol<'s, 'm, N, T> {
    pub fn new(matrixes: &'s [&'m Matrix<N, T>]) -> Self {
        Self { matrixes }
    }

    pub fn get_matrix(&self, index: usize) -> Option<&'m Matrix<N, T>> {
        Some(*self.matrixes.get(index)?)
    }
}

impl<'c, 'm, const N: usize, T> MatrixCol<'c, 'm, N, T>
where
    T: Add<Output = T> + Copy,
{
    pub fn sum(&self) -> Option<T> {
        let mut sum: Option<T> = None;
        for m in self.matrixes {
            if let Some(m_sum) = m.sum() {
                match sum {
                    Some(s) => sum = Some(s + m_sum),
                    None => sum = Some(m_sum),
                }
            }
        }
        sum
    }
}

impl<'c, 'm, const N: usize, T> MatrixCol<'c, 'm, N, T>
where
    T: Mul<Output = T> + Copy,
{
    pub fn prod(&self) -> Option<T> {
        let mut prod: Option<T> = None;
        for &m in self.matrixes {
            if let Some(m_sum) = m.prod() {
                match prod {
                    Some(s) => prod = Some(s * m_sum),
                    None => prod = Some(m_sum),
                }
            }
        }
        prod
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_add() {
        let mut matrix = Matrix::<3, usize>::from_vec(vec![1, 2, 3]).unwrap();
        matrix = matrix + 3;
        assert_eq!(matrix.elements, vec![4, 5, 6]);
    }

    #[test]
    fn test_matrix_mul() {
        let mut matrix = Matrix::<3, usize>::from_vec(vec![1, 2, 3]).unwrap();
        matrix = matrix * 3;
        assert_eq!(matrix.elements, vec![3, 6, 9]);
    }

    #[test]
    fn test_matrix_col_get_matrix() {
        let m_1 = Matrix::<3, usize>::from_vec(vec![1, 2, 3]).unwrap();
        let m_2 = Matrix::<3, usize>::from_vec(vec![1, 2, 3]).unwrap();
        let m_3 = Matrix::<3, usize>::from_vec(vec![1, 2, 3]).unwrap();
        let binding = [&m_1, &m_2, &m_3];
        let m_col = MatrixCol::new(&binding);

        for (i, &m) in binding.iter().enumerate() {
            assert_eq!(m_col.get_matrix(i).unwrap(), m);
        }
    }

    #[test]
    fn test_matrix_col_sum() {
        let m_1 = Matrix::<3, usize>::from_vec(vec![1, 1, 1]).unwrap();
        let m_2 = Matrix::<3, usize>::from_vec(vec![2, 2, 2]).unwrap();
        let m_3 = Matrix::<3, usize>::from_vec(vec![3, 3, 3]).unwrap();
        let binding = [&m_1, &m_2, &m_3];
        let m_col = MatrixCol::new(&binding);

        assert_eq!(m_col.sum().unwrap(), 18);
    }

    #[test]
    fn test_matrix_col_prod() {
        let m_1 = Matrix::<3, usize>::from_vec(vec![1, 1, 1]).unwrap();
        let m_2 = Matrix::<3, usize>::from_vec(vec![2, 2, 2]).unwrap();
        let m_3 = Matrix::<3, usize>::from_vec(vec![3, 3, 3]).unwrap();
        let binding = [&m_1, &m_2, &m_3];
        let m_col = MatrixCol::new(&binding);

        assert_eq!(m_col.prod().unwrap(), 216);
    }

    #[test]
    fn test_matrix_lifetime() {
        let m_1 = Matrix::<3, usize>::from_vec(vec![1, 1, 1]).unwrap();
        let m_2 = Matrix::<3, usize>::from_vec(vec![2, 2, 2]).unwrap();
        let m_3 = Matrix::<3, usize>::from_vec(vec![3, 3, 3]).unwrap();
        let binding = [&m_1, &m_2, &m_3];
        {
            let _ = MatrixCol::new(&binding);
        }
    }
}
