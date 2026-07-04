use macuru::adt;

adt!(
    Data = Empty | NonEmpty with DataFunc {
        fn add(&self, v: usize) -> Option<Self>;
        fn remove(&self, v: usize) -> Option<Self>;
    }
);

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Empty(String);

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct NonEmpty {
    name: String,
    value: usize,
}

impl DataFunc for Empty {
    fn add(&self, v: usize) -> Option<Data> {
        if v > 0 {
            Some(
                NonEmpty {
                    name: self.0.clone(),
                    value: v,
                }
                .into(),
            )
        } else {
            None
        }
    }

    fn remove(&self, _v: usize) -> Option<Data> {
        None
    }
}

impl DataFunc for NonEmpty {
    fn add(&self, v: usize) -> Option<Data> {
        Some(
            Self {
                name: self.name.clone(),
                value: self.value + v,
            }
            .into(),
        )
    }

    fn remove(&self, v: usize) -> Option<Data> {
        if self.value > v {
            Some(
                Self {
                    name: self.name.clone(),
                    value: self.value - v,
                }
                .into(),
            )
        } else if self.value == v {
            Some(Empty(self.name.clone()).into())
        } else {
            None
        }
    }
}

fn main() {
    let d1: Data = Empty("data1".to_string()).into();
    println!("d1 = {:?}", d1);

    println!("d1 add(0) = {:?}", d1.add(0));

    let d2 = d1.add(3).unwrap();
    println!("d2 = {:?}", d2);

    let d3 = d2.add(2).unwrap();
    println!("d3 = {:?}", d3);

    println!("d3 remove(6) = {:?}", d3.remove(6));

    let d4 = d3.remove(4).unwrap();
    println!("d4 = {:?}", d4);

    let d5 = d4.remove(1).unwrap();
    println!("d5 = {:?}", d5);

    println!("d5 remove(1) = {:?}", d5.remove(1));
}
