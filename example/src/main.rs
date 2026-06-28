use frsums::adt;

adt!(Data = Elem1 | Elem2);

#[derive(Clone, Debug)]
pub struct Elem1(isize);

#[derive(Clone, Debug)]
pub struct Elem2 {
    name: String,
    value: isize,
}

fn main() {
    let d1 = Data::Elem1_(Elem1(12));
    println!("{:?}", d1);

    let d2 = Data::Elem2_(Elem2 { name: "abc".into(), value: 34 });
    println!("{:?}", d2);
}
