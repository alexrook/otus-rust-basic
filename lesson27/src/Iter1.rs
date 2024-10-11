#[cfg(test)]
mod tests {

    #[test]
    fn check1() {
        let arr = [1, 2, 3];
        let iter = arr.into_iter(); // Получаем итератор

        for value in iter {
            println!("{}", value); // Печатает 1, 2, 3
        }

        println!("{:?}", arr)
    }
}
