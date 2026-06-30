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
}
