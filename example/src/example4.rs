use macuru::adt;

adt!(
    Tree<V: Copy> = Leaf<V> | Node<V> with TreeFunc<V> {
        fn values(&self) -> Vec<V>;
    }
);

type Node<V> = GenericNode<Box<Tree<V>>>;

pub struct Leaf<V>(V);

pub struct GenericNode<T> {
    left: T,
    right: Option<T>,
}

impl<V: Copy> TreeFunc<V> for Leaf<V> {
    fn values(&self) -> Vec<V> {
        vec![self.0]
    }
}

impl<V: Copy> TreeFunc<V> for Node<V> {
    fn values(&self) -> Vec<V> {
        let vl = self.left.values();
        let vr = self.right.as_ref().map(|x| x.values()).unwrap_or_default();

        [vl, vr].concat()
    }
}

fn main() -> Result<(), ()> {
    type TreeU = Tree<usize>;

    let leaf = |v: usize| -> TreeU { Leaf(v).into() };

    let node = |left: TreeU, right: Option<TreeU>| -> TreeU {
        Node {
            left: left.into(),
            right: right.map(|x| x.into()),
        }
        .into()
    };

    let t1: TreeU = Node {
        left: leaf(1).into(),
        right: Some(leaf(2).into()),
    }
    .into();

    println!("t1={:?}", t1.values());

    let t2: TreeU = node(
        node(
            node(node(leaf(1), None), Some(node(leaf(2), Some(leaf(3))))),
            Some(node(leaf(40), Some(leaf(50)))),
        ),
        Some(node(node(leaf(210), Some(leaf(220))), None)),
    );

    println!("t2={:?}", t2.values());

    Ok(())
}
