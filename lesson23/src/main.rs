use std::ops::{Add, AddAssign};

pub struct Foo(u64);

impl Add for Foo {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Foo(self.0 + other.0)
    }
}

impl Add<u64> for Foo {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Foo(self.0 + rhs)
    }
}

impl Add<Foo> for u64 {
    type Output = Self;
    fn add(self, rhs: Foo) -> Self::Output {
        self + rhs.0
    }
}

impl AddAssign for Foo {
    fn add_assign(&mut self, rhs: Self) {
        //*self = Foo(self.0 + rhs.0)
        self.0 += rhs.0
    }
}

impl AddAssign<u64> for Foo {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0 + rhs
    }
}

fn main() {
    println!("It compiles")
}