//! Занятие 9
//! Домашнее задание
//!    Требования:
//! * Убедиться, что копилятор не позволит вернуть
//!     более одной мутабельной ссылки на один объект.
//! * Реализованы и протестированы все перечисленные функции.
//! * `cargo clippy`` и `cargo fmt --check` не выдают предупреждений и ошибок.

/// Принимает мутабельную ссылку на кортеж и bool значение.
/// * Если false, возвращает мутабельную ссылку на первый элемент кортежа.
/// * Если true, возвращает мутабельную ссылку на второй элемент кортежа.
pub fn get_elem<A, B>((a, b): &mut (A, B), flag: bool) -> Result<&mut A, &mut B> {
    if flag {
        Ok(a)
    } else {
        Err(b)
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
    println!("index[{}], n[{}], len[{}]", i, n, slice.len());
    &mut slice[i]
}

///Принимает слайс и число N. Возвращает два слайса с элементами:
///с нулевого по N-1;  с N-го по последний;
pub fn split_slice<T>(slice: &[T], n: usize) -> (&[T], &[T]) {
    let first: &[T] = &slice[0..n];
    let second: &[T] = &slice[n..slice.len()];
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

    use super::*;

    #[test]
    fn get_elem_should_return_ok() {
        let mut tuple = ("e".to_string(), 2);
        let actual = get_elem(&mut tuple, true);

        assert_eq!(actual, Ok(&mut "e".to_string()));
    }

    #[test]
    fn get_elem_should_return_err() {
        let mut tuple = ("e".to_string(), 2);
        let actual = get_elem(&mut tuple, false);

        assert_eq!(actual, Err(&mut 2));
    }

    #[test]
    fn get_nth_must_return_correct_value() {
        let mut a = [1, 2, 3, 4, 5];
        let slice1 = &mut a[1..3];
        let slice2 = &mut [1, 2, 3, 4, 5];
        let actual = get_nth(slice1, 3);
        let actual2 = get_nth(slice2, 3);
        assert_eq!(actual, &mut 4)
    }

    #[test]
    fn get_nth_reverse_return_correct_value() {
        let slice = &mut [1, 2, 3, 4, 5];
        let actual = get_nth_reverse(slice, 4);
        assert_eq!(actual, &mut 2)
    }
}
