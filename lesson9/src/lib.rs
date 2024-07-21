//! Занятие 9
//! Домашнее задание
//!    Требования:
//! * Убедиться, что компилятор не позволит вернуть
//!     более одной мутабельной ссылки на один объект.
//! * Реализованы и протестированы все перечисленные функции.
//! * `cargo clippy`` и `cargo fmt --check` не выдают предупреждений и ошибок.

#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: PartialEq, B: PartialEq> PartialEq for Either<A, B> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Either::Left(a) => {
                if let Either::Left(a1) = other {
                    a == a1
                } else {
                    false
                }
            }
            Either::Right(b) => {
                if let Either::Right(b1) = other {
                    b == b1
                } else {
                    false
                }
            }
        }
    }
}
/// Принимает мутабельную ссылку на кортеж и bool значение.
/// * Если false, возвращает мутабельную ссылку на первый элемент кортежа.
/// * Если true, возвращает мутабельную ссылку на второй элемент кортежа.
pub fn get_elem<A, B>((a, b): &mut (A, B), flag: bool) -> Either<&mut A, &mut B> {
    if flag {
        Either::Left(a)
    } else {
        Either::Right(b)
    }
}

/// Принимает мутабельную ссылку на слайс и число N.
/// Возвращает мутабельную ссылку на N-ый элемент.
pub fn get_nth<T>(slice: &mut [T], n: usize) -> &mut T {
    &mut slice[n]
}

/// Принимает слайс и число N.
/// Возвращает ссылку на N-ый элемент слайса с конца.
pub fn get_nth_reverse<T>(slice: &mut [T], n: usize) -> &mut T {
    let i: usize = slice.len() - n;
    //println!("index[{}], n[{}], len[{}]", i, n, slice.len());
    &mut slice[i]
}

///Принимает слайс и число N. Возвращает два слайса с элементами:
///с нулевого по N-1;  с N-го по последний;
pub fn split_slice<T>(slice: &[T], n: usize) -> (&[T], &[T]) {
    let first: &[T] = &slice[..n];
    let second: &[T] = &slice[n..];
    (first, second)
}

///Принимает слайс и возвращает массив слайсов,
///содержащий четыре равные (насколько возможно) части исходного слайса.
pub fn get_slice_array<T>(slice: &[T]) -> [&[T]; 4] {
    let len: usize = slice.len();
    let part_size: usize = len / 4;
    [
        &slice[0..part_size],
        &slice[part_size..(part_size * 2)],
        &slice[(part_size * 2)..(part_size * 3)],
        &slice[(part_size * 3)..len],
    ]
}
//     Протестировать функции.

#[cfg(test)]
pub mod test {

    use crate::split_slice;

    use super::*;

    #[test]
    fn get_elem_should_return_ok() {
        let mut tuple = ("e".to_string(), 2);
        let actual = get_elem(&mut tuple, true);

        assert_eq!(actual, Either::Left(&mut "e".to_string()));
    }

    #[test]
    fn get_elem_should_return_err() {
        let mut tuple = ("e".to_string(), 2);
        let actual = get_elem(&mut tuple, false);

        assert_eq!(actual, Either::Right(&mut 2));
    }

    #[test]
    fn get_nth_must_return_correct_value() {
        let slice1 = &mut [2, 3];
        //индексация с 0 для 2-х элементов
        let actual = get_nth(slice1, 1);
        assert_eq!(*actual, 3);
        //еще
        let slice2 = &mut [1, 2, 3, 4, 5];
        let actual2 = get_nth(slice2, 3);
        assert_eq!(*actual2, 4); //индексация с 0, на позиции[3] элемент[4]
    }

    #[test]
    fn get_nth_reverse_return_correct_value() {
        let slice = &mut [1, 2, 3, 4, 5];
        let actual = get_nth_reverse(slice, 4);
        assert_eq!(actual, &mut 2)
    }

    #[test]
    fn split_slice_should_work_correctly() {
        let slice: &[i32; 5] = &[1, 2, 3, 4, 5];
        let (first, second) = split_slice(slice, 2);
        assert_eq!(first, [1, 2]);
        assert_eq!(second, [3, 4, 5]);
    }

    #[test]
    fn get_slice_array_should_work_correctly() {
        //четного размера массив
        let even_size_slice = &[1, 2, 3, 4];
        let actual1 = get_slice_array(even_size_slice);
        assert_eq!(actual1, [[1], [2], [3], [4]]);

        //нечетного размера массив
        let odd_size_slice = &[1, 2, 3, 4, 5];
        let actual2 = get_slice_array(odd_size_slice);
        //для слайса из 5 элементов,результат таков, что:
        assert_eq!(actual2[0..=2], [[1], [2], [3]]); //в первых трех элементах результата по одноразмерному слайсу
        assert_eq!(actual2[3], [4, 5]); //в последнем  2
    }
}
