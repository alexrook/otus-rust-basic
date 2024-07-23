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
    let i: usize = slice.len() - 1 - n;
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
    let mut ret: [&[T]; 4] = [&[]; 4];
    let len: usize = slice.len();
    let part_size: usize = if len / 4 > 0 { len / 4 } else { 1 };

    for (i, chunk) in slice.chunks(part_size).enumerate() {
        if i > 3 {
            let remider = len % 4;
            let pos = len - remider - part_size;
            ret[3] = &slice[pos..]
        } else {
            ret[i] = chunk
        }
    }

    ret
}

// Протестировать функции.
#[cfg(test)]
pub mod test {

    use crate::split_slice;

    use super::*;

    #[test]
    fn get_elem_should_return_left() {
        let mut tuple = ("e".to_string(), 2);
        let actual = get_elem(&mut tuple, true);

        assert_eq!(actual, Either::Left(&mut "e".to_string()));
    }

    #[test]
    fn get_elem_should_return_right() {
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
        let slice2: &mut [i32; 5] = &mut [1, 2, 3, 4, 5];
        let actual2 = get_nth(slice2, 3);
        assert_eq!(*actual2, 4); //индексация с 0, на позиции[3] элемент[4]
                                 //еще
    }

    #[test]
    #[should_panic]
    fn get_nth_should_panic_for_empty_arrays() {
        let slice: &mut [i32; 0] = &mut [];
        let _ = get_nth(slice, 0);
    }

    #[test]
    #[should_panic]
    fn get_nth_should_panic_for_bad_index() {
        let slice: &mut [i32; 1] = &mut [1];
        let _ = get_nth(slice, 2);
    }

    #[test]
    fn get_nth_reverse_return_correct_value() {
        let slice = &mut [1, 2, 3, 4, 5];

        assert_eq!(*get_nth_reverse(slice, 0), 5);
        assert_eq!(*get_nth_reverse(slice, 1), 4);
        assert_eq!(*get_nth_reverse(slice, 2), 3);
        assert_eq!(*get_nth_reverse(slice, 3), 2);
        assert_eq!(*get_nth_reverse(slice, 4), 1);
    }

    #[test]
    #[should_panic]
    fn get_nth_reverse_should_panic_when_index_is_greater_then_length() {
        let slice = &mut [1, 2, 3, 4, 5];
        assert_eq!(*get_nth_reverse(slice, 5), 5); //Boom
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

        let small_size_slice = &[1, 2, 3];

        let actual3 = get_slice_array(small_size_slice);
        assert_eq!(actual3[0..=2], [[1], [2], [3]]); //в первых трех элементах результата по одноразмерному слайсу
        assert_eq!(actual3[3], [0; 0]);
        //println!("{:?}", actual3);

        let big_size_slice = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let actual4 = get_slice_array(big_size_slice);
        println!("{:?}", actual4);

        let large_size_slice = &[1; 101];
        let actual5 = get_slice_array(large_size_slice);
        println!("{:?}", actual5);
    }
}
