///
/// функция double_int32 принимает 32-х битное целое знаковое число
/// и возвращает 32-х битное целое знаковое число, равное удвоенному входному.
fn double_int32(a: i32) -> i32 {
    a * 2
}
///
///  функция double_int64 принимает 32-х битное целое знаковое число
/// и возвращает 64-х битное целое знаковое число, равное удвоенному входному.
fn double_int64(a: i32) -> i64 {
    a as i64 * 2
}
///
/// функция double_float32 принимает 32-х битное число с плавающей точкой
/// и возвращает 32-х битное число с плавающей точкой, равное удвоенному входному.
fn double_float32(a: f32) -> f32 {
    a * 2.0
}
///
/// функция double_float64 принимает 32-х битное число с плавающей точкой
/// и возвращает 64-х битное число с плавающей точкой, равное удвоенному входному.
fn double_float64(a: f32) -> f64 {
    a as f64 * 2.0
}
///
/// функция int_plus_float_to_float принимает 32-х битное целое знаковое число и 32-х битное число с плавающей точкой.
/// Возвращает 64-х битное число с плавающей точкой, равное сумме входных.
fn int_plus_float_to_float(a: i32, b: f32) -> f64 {
    a as f64 + b as f64
}
///
///  функция int_plus_float_to_int принимает 32-х битное целое знаковое число и 32-х битное число с плавающей точкой.
/// Возвращает 64-х битное целое знаковое число, равное сумме входных.
fn int_plus_float_to_int(a: i32, b: f32) -> i64 {
    a as i64 + b as i64
}
///
///  функция tuple_sum принимает кортеж из двух целых чисел.
/// Возвращает целое число, равное сумме чисел во входном кортеже.
///
fn tuple_sum_v0(a: (u32, u32)) -> u32 {
    a.0 + a.1
}

fn tuple_sum_v1(a: (u32, u32)) -> u32 {
    let (left, right) = a;
    left + right
}
//pattern matching in args
fn tuple_sum_v2((left, right): (u32, u32)) -> u32 {
    left + right
}

///
///  функция array_sum принимает массив из трёх целых чисел.
/// Возвращает целое число, равное сумме чисел во входном массиве.
fn array_sum_v1(a: [u32; 3]) -> u32 {
    //i'm in the process of learning
    #[allow(clippy::unnecessary_fold)]
    a.iter().fold(0, |acc, x| acc + x)
}

fn array_sum_v1a(a: [u32; 3]) -> u32 {
    a.iter().sum()
}

fn array_sum_v2(a: [u32; 3]) -> u32 {
    let mut sum = 0;
    for x in a {
        sum += x;
    }

    sum
}

fn array_sum_v3(a: [u32; 3]) -> u32 {
    let mut sum = 0;
    let mut idx = 0;
    while idx < a.len() {
        sum += a[idx];
        idx += 1;
    }

    sum
}

//FIXME: please explain how I can run loop1
// fn array_sum_v4(a: [u32; 3]) -> u32 {
//     let mut loop1 = |idx:usize, acc:u32| {
//         if idx < 3 {
//             loop1(idx + 1, acc + a[idx]) //Error
//         } else {
//             acc
//         }
//     };
//     loop1(0, 0)
// }
fn main() {
    println!("double_int32:{}", double_int32(-12));
    println!("double_int64:{}", double_int64(-22));
    println!("double_float32:{}", double_float32(-22.01_f32));
    println!("double_float6432:{}", double_float64(-32.02_f32));

    println!(
        "int_plus_float_to_float:{}",
        int_plus_float_to_float(42, 0.2_f32)
    );

    println!(
        "int_plus_float_to_int:{}",
        int_plus_float_to_int(42, 1.2_f32)
    );

    println!("tuple_sum_v0:{}", tuple_sum_v0((23, 45)));
    println!("tuple_sum_v2:{}", tuple_sum_v2((23, 45)));
    println!("tuple_sum_v1:{}", tuple_sum_v1((23, 45)));
    println!("tuple_sum_v2:{}", tuple_sum_v2((23, 45)));

    println!("array_sum_v1:{}", array_sum_v1([1, 2, 3]));
    println!("array_sum_v1a:{}", array_sum_v1a([1, 2, 3]));
    println!("array_sum_v2:{}", array_sum_v2([1, 2, 3]));
    println!("array_sum_v3:{}", array_sum_v3([1, 2, 3]));
}
