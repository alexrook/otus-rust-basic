use lesson19::even_func_name;

fn main() {
    let r = even_func_name!(fo(), foo, bar, one_more_time_);
    println!("{:?}", r);
}

pub fn fo() -> u32 {
    42
}

pub fn foo() -> u32 {
    43
}

pub fn bar() -> String {
    "42".to_owned()
}

pub fn one_more_time_() -> String {
    "Never again is what you swore the time before".to_owned()
}

#[macro_export]
macro_rules! say_hello {
    () => {
        println!("Hello, world!");
    };
}

#[macro_export]
macro_rules! tupled {
    ($( $x:expr ),*) => {
        (
            $(
                $x(),
            )*
        )
    };
}

#[cfg(test)]
mod tests {

    use lesson19::*;

    #[test]
    fn add_one_should_work_with_fn() {
        let r = add_one!(41);
        assert_eq!(r, 42)
    }

    #[test]
    fn even_func_name_should_work_with_fn() {
        let (ret_fo, ret_omt) = even_func_name!(fo, foo, bar, one_more_time_);
        assert_eq!(ret_fo, 42);
        assert_eq!(ret_omt, "It's amaizing");
    }

    #[test]
    fn tupled_should_work_with_fn() {
        let (r1, r2, r3) = tupled!(fo, bar, again);
        assert_eq!(r1, 42);
        assert_eq!(r2, "42");
        assert_eq!(r3.len(), 45);
    }

    fn fo() -> u32 {
        42
    }

    fn bar() -> String {
        "42".to_owned()
    }

    fn again() -> String {
        "Never again is what you swore the time before".to_owned()
    }

    pub fn one_more_time_() -> String {
        "It's amaizing".to_owned()
    }
}
