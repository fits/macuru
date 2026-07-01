use frsums::adt;

adt!(Data = Elem1 | Elem2);

#[derive(Clone, Debug)]
pub struct Elem1(isize);

#[derive(Clone, Debug)]
pub struct Elem2 {
    name: String,
    value: isize,
}

adt!(
    Item = A | B with {
        fn show(&self) -> String;
        fn print(&self, prefix: &str, value: i32, flag: bool);
    }
);

#[derive(Clone, Debug)]
pub struct A {
    name: String,
}

#[derive(Clone, Debug)]
pub struct B {
    name: String,
    value: isize,
}

impl ItemImpl for A {
    fn show(&self) -> String {
        format!("-A- : {:?}", self)
    }

    fn print(&self, prefix: &str, value: i32, flag: bool) {
        println!("{prefix} {value} {flag}, {self:?}");
    }
}

impl ItemImpl for B {
    fn show(&self) -> String {
        format!("+B+ : {:?}", self)
    }

    fn print(&self, prefix: &str, value: i32, flag: bool) {
        println!("{prefix} {value} {flag}, B(name={}, value={})", self.name, self.value);
    }
}

fn main() {
    let d1 = Data::Elem1_(Elem1(12));
    println!("{:?}", d1);

    let d2 = Data::Elem2_(Elem2 {
        name: "abc".into(),
        value: 34,
    });
    println!("{:?}", d2);

    let d3: Data = Elem1(5).into();
    println!("{:?}", d3);

    let d4: Data = Elem2 {
        name: "def".into(),
        value: 7,
    }
    .into();
    println!("{:?}", d4);

    println!("Elem1 to Elem1 = {:?}", Elem1::try_from(d3.clone()));
    println!("Elem1 to Elem2 = {:?}", Elem2::try_from(d3.clone()));
    println!("Elem2 to Elem1 = {:?}", Elem1::try_from(d4.clone()));
    println!("Elem2 to Elem2 = {:?}", Elem2::try_from(d4.clone()));

    test_item();
}

fn test_item() {
    let d1: Item = A {
        name: "item-1".to_string(),
    }
    .into();

    println!("d1.show = {}", d1.show());
    d1.print("* test1", 12, true);

    let d2: Item = B {
        name: "item-2".to_string(),
        value: 123,
    }
    .into();

    println!("d2.show = {}", d2.show());
    d2.print("* test2", 345, false);
}
