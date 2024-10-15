use std::any::Any;

pub trait AAA: Any {
    fn as_any(&self) -> &dyn Any;
}

impl dyn AAA {
    pub fn print(&self) {
        if let Some(_) = self.as_any().downcast_ref::<CCC>() {
            println!("Its CCC")
        } else if let Some(_) = self.as_any().downcast_ref::<BBB>() {
            println!("Its BBB")
        } else {
            println!("Unknown Type")
        }
    }

    pub fn new(v: impl AAA) -> Box<dyn AAA> {
        Box::new(v)
    }
}

struct BBB;
impl AAA for BBB {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
struct CCC;
impl AAA for CCC {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let aaa = <dyn AAA>::new(BBB);
        <dyn AAA>::print(&*aaa);
        (&*aaa as &dyn AAA).print()
    }
}
