use lesson25::{Cat, Pet};

fn main() -> Result<(), &'static str> {
    let smelly_cat: Cat = Cat::new("Smelly cat".to_string(), 3);

    show_info(&smelly_cat); //Предоставлять ссылку на строковый слайс - имя кота

    let mut smelly_cat_older: Cat = smelly_cat + 5;
    println!(
        "During the journey, the cat[{}] managed to grow up",
        smelly_cat_older
    );

    smelly_cat_older += 7;

    println!(
        "During the journey, the cat[{}] managed to grow up again",
        smelly_cat_older
    );

    let cat_pet: Pet = Pet::Cat(Cat::new("Cheshire".to_string(), 120));

    let cheshire_cat: Result<Cat, &'static str> = cat_pet.try_into();

    println!("Cheshire cat[{}]", cheshire_cat?); //Display trait is used here

    let dog_pet: Pet = Pet::Dog {
        name: "Barbas".to_string(),
        age: 1200,
    };

    let unsuccessfull_try_into_cat: Result<Cat, &str> = dog_pet.try_into();

    let err_str: &str =
        unsuccessfull_try_into_cat.expect_err("There is something wrong with your code");

    println!("{}, It's ok really", err_str);

    Ok(())
}

fn show_info<T: AsRef<str>>(value: &T) {
    println!("Info[{}]", value.as_ref())
}
