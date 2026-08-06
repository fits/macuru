use macuru::{MonadLike, mdo};

fn main() -> Result<(), ()> {
    let d1 = mdo!(
        x <- vec![1, 2]
        yield x + 1
    );

    println!("d1 = {:?}", d1);

    let v1 = vec![1, 3, 5];

    let d2 = mdo!(
            x <- vec!["a", "b"]
            y <- v1.clone()
            yield (x, y * 2)
    );

    println!("d2 = {:?}", d2);

    let f = |v: i32| if v > 3 { Some(v) } else { None };

    let d3 = mdo!(
        x <- Some("a")
        y <- Some("b")
        z <- f(5)
        yield format!("{}-{}-{}", x, y, z)
    );

    println!("d3 = {:?}", d3);

    let d4 = mdo!(
        a <- f(10)
        b <- f(3)
        yield (a, b)
    );

    println!("d4 = {:?}", d4);

    let d5 = mdo!(
        a <- vec![1, 5]
        b <- mdo!(
            x <- vec!["a", "d"]
            y <- vec![true, false]
            yield (x, y)
        )
        yield (a + 10, b)
    );

    println!("d5 = {:?}", d5);

    Ok(())
}
