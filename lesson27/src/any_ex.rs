use std::any::Any;

pub trait Test1: Any {
    fn as_any(&self) -> &dyn Any;
}

impl dyn Test1 {
    pub fn print(&self) {
        if self.as_any().downcast_ref::<Test3>().is_some() {
            println!("Its CCC")
        } else if self.as_any().downcast_ref::<Test2>().is_some() {
            println!("Its BBB")
        } else {
            println!("Unknown Type")
        }
    }

    pub fn new(v: impl Test1) -> Box<dyn Test1> {
        Box::new(v)
    }
}

struct Test2;
impl Test1 for Test2 {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
struct Test3;
impl Test1 for Test3 {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let aaa = <dyn Test1>::new(Test2);
        <dyn Test1>::print(&*aaa);
        (&*aaa as &dyn Test1).print()
    }
}
