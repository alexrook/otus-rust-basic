use lesson9::get_nth;

//    Убедиться, что компилятор не позволит вернуть
//     более одной мутабельной ссылки на один объект.
fn main() {
    let slice = &mut [1, 2, 3, 4, 5];
    let actual = get_nth(slice, 4);
    println!("{}", actual);
    //Неявный drop actual здесь ? ->
    //Да. Компилятор видит, что actual не используется, после let actual2 = ...,
    //и уничтожает ссылку, чтобы можно было повторно одолжить slice

    let actual2 = get_nth(slice, 4);
    println!("{}", actual2);

    //Если попытаться вызвать actual еше раз возникнет ошибка
    //     error[E0499]: cannot borrow `*slice` as mutable more than once at a time
    //   --> lesson9/src/main.rs:10:27
    //    |
    // 5  |     let actual = get_nth(slice, 4);
    //    |                          ----- first mutable borrow occurs here
    // ...
    // 10 |     let actual2 = get_nth(slice, 4);
    //    |                           ^^^^^ second mutable borrow occurs here
    // ...
    // 15 |     println!("{}", actual);
    //    |                    ------ first borrow later used here
    //println!("{}", actual);
}
