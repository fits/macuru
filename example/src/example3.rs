use macuru::adt;

use std::fmt::Debug;
use std::ops::Add;

adt!(
    Stock<ID, QTY, INFO> = EmptyStock<ID> | NonEmptyStock<ID, QTY> derive Clone, Debug
    with StockFunc<ID, QTY, INFO>
    where
        ID: Debug + Clone,
        QTY: Copy + Default + PartialOrd + Add<Output = QTY>,
        INFO: Debug,
    {
        fn id(&self, info: INFO) -> ID;
        fn qty(&self, info: INFO) -> QTY;
        fn restock(&self, q: QTY, info: INFO) -> Option<Self>;
    }
);

#[derive(Clone, Debug)]
pub struct EmptyStock<ID>(ID);

#[derive(Clone, Debug)]
pub struct NonEmptyStock<ID, QTY> {
    id: ID,
    qty: QTY,
}

impl<ID, QTY, INFO> StockFunc<ID, QTY, INFO> for EmptyStock<ID>
where
    ID: Debug + Clone,
    QTY: Copy + Default + PartialOrd + Add<Output = QTY>,
    INFO: Debug,
{
    fn id(&self, info: INFO) -> ID {
        println!("* empty id info: {:?}", info);
        self.0.clone()
    }

    fn qty(&self, info: INFO) -> QTY {
        println!("* empty qty info: {:?}", info);
        QTY::default()
    }

    fn restock(&self, q: QTY, info: INFO) -> Option<Stock<ID, QTY>> {
        println!("* empty restock info: {:?}", info);

        if q > QTY::default() {
            Some(
                NonEmptyStock {
                    id: self.0.clone(),
                    qty: q,
                }
                .into(),
            )
        } else {
            None
        }
    }
}

impl<ID, QTY, INFO> StockFunc<ID, QTY, INFO> for NonEmptyStock<ID, QTY>
where
    ID: Debug + Clone,
    QTY: Copy + Default + PartialOrd + Add<Output = QTY>,
    INFO: Debug,
{
    fn id(&self, info: INFO) -> ID {
        println!("% nonempty id info: {:?}", info);
        self.id.clone()
    }

    fn qty(&self, info: INFO) -> QTY {
        println!("% nonempty qty info: {:?}", info);
        self.qty
    }

    fn restock(&self, q: QTY, info: INFO) -> Option<Stock<ID, QTY>> {
        println!("% nonempty restock info: {:?}", info);

        if q > QTY::default() {
            Some(
                Self {
                    id: self.id.clone(),
                    qty: self.qty + q,
                }
                .into(),
            )
        } else {
            None
        }
    }
}

fn main() -> Result<(), ()> {
    type Stock1 = Stock<String, usize>;

    let print_stock = |s: &Stock1| {
        println!(
            "stock id={}, qty={}, {:?}",
            s.id("print_stock"),
            s.qty("print_stock"),
            s
        );
    };

    let s1: Stock1 = EmptyStock("stock-1".to_string()).into();
    print_stock(&s1);

    if let Some(s2) = s1.restock(1, 11) {
        print_stock(&s2);

        if let Some(s3) = s2.restock(2, 12) {
            print_stock(&s3);
        }
    }

    println!("restock 0: {:?}", s1.restock(0, "test"));

    Ok(())
}
