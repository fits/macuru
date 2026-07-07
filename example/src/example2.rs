use macuru::adt;

adt!(
    Data = A | B with ShowData {
        fn show(&self) -> String;
    }
);

pub struct A(i32);
pub struct B(String);

impl A {
    fn add(&self, v: i32) -> Self {
        Self(self.0 + v)
    }

    fn to_b(&self) -> B {
        B(format!("{}", self.0))
    }
}

impl ShowData for A {
    fn show(&self) -> String {
        format!("A(value={})", self.0)
    }
}

impl ShowData for B {
    fn show(&self) -> String {
        format!("B(value={})", self.0)
    }
}

fn main() -> Result<(), ()> {
    let d1: Data = A(12).into();
    println!("d1={}", d1.show());

    let a1 = A::try_from(d1)?;
    println!("a1={}", a1.show());

    let a2 = a1.add(34);
    println!("a2={}", a2.show());

    let d2: Data = a2.to_b().into();
    println!("d2={}", d2.show());
    println!("d2 A::try_from: error={}", A::try_from(d2).is_err());

    Ok(())
}
