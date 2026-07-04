[japanese](./README_ja.md)

# Macuru

```Macuru``` is the utility macro library for Rust.

## Usage

```toml
[dependencies]
macuru = { git = "https://github.com/fits/macuru" }
```

```adt!``` macro generates boilerplate code for ADT.

```rust
use macuru::adt;

adt!(
    Data = Empty | NonEmpty with DataFunc {
        fn add(&self, v: usize) -> Option<Self>;
        fn remove(&self, v: usize) -> Option<Self>;
    }
);

#[derive(Clone, Debug)]
pub struct Empty(String);

#[derive(Clone, Debug)]
pub struct NonEmpty {
    name: String,
    value: usize,
}

impl DataFunc for Empty {
    fn add(&self, v: usize) -> Option<Data> {
        ...
    }
    ...
}
...
```

## ADT (Algebraic data type) Macro

```adt!``` macro has the following features to assist with ADT definitions in Rust.

* generate the enum type
    * element name is ```<type>_```
* implement enum and element type conversion
* definition and implement for trait
    * convert ```Self``` return type into the enum type

```rust
adt!(
    <enum-type> = <type> | <type> ... [ with <trait-name> {
        <trait-function>;
        ...
    }]
);
```

However, there are the following restrictions.

* method receiver is only ```&self```
* generics are not permitted for types or trait

### Example1

```rust
adt!( Data = Elem1 | Elem2 );
```

#### Macro results

```rust
#[derive(Clone, Debug)]
pub enum Data {
    Elem1_(Elem1),
    Elem2_(Elem2),
}

impl From<Elem1> for Data {
    fn from(v: Elem1) -> Self {
        Self::Elem1_(v)
    }
}

impl TryFrom<Data> for Elem1 {
    type Error = ();

    fn try_from(v: Data) -> Result<Self, Self::Error> {
        if let Data::Elem1_(x) = v {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl From<Elem2> for Data {
    fn from(v: Elem2) -> Self {
        Self::Elem2_(v)
    }
}

impl TryFrom<Data> for Elem2 {
    type Error = ();

    fn try_from(v: Data) -> Result<Self, Self::Error> {
        if let Data::Elem2_(x) = v {
            Ok(x)
        } else {
            Err(())
        }
    }
}
```

### Example2

```rust
adt!( 
    Data = Elem1 | Elem2 with DataFunc {
        fn func1(&self);
        fn func2(&self, a: isize) -> Self;
        fn func3(&self, a: String, b: bool) -> (Self, isize);
        fn func4(&self, a: f32) -> Result<(Self, String, isize), ()>;
    }
);
```

#### Macro results

```rust
#[derive(Clone, Debug)]
pub enum Data {
    Elem1_(Elem1),
    Elem2_(Elem2),
}

pub trait DataFunc {
    fn func1(&self);
    fn func2(&self, a: isize) -> Data;
    fn func3(&self, a: String, b: bool) -> (Data, isize);
    fn func4(&self, a: f32) -> Result<(Data, String, isize), ()>;
}

impl DataFunc for Data {
    fn func1(&self) {
        match self {
            Self::Elem1_(x) => DataFunc::func1(x),
            Self::Elem2_(x) => DataFunc::func1(x),
        }
    }

    fn func2(&self, a: isize) -> Data {
        match self {
            Self::Elem1_(x) => DataFunc::func2(x, a),
            Self::Elem2_(x) => DataFunc::func2(x, a),
        }
    }

    fn func3(&self, a: String, b: bool) -> (Data, isize) {
        match self {
            Self::Elem1_(x) => DataFunc::func3(x, a, b),
            Self::Elem2_(x) => DataFunc::func3(x, a, b),
        }
    }

    fn func4(&self, a: f32) -> Result<(Data, String, isize), ()> {
        match self {
            Self::Elem1_(x) => DataFunc::func4(x, a),
            Self::Elem2_(x) => DataFunc::func4(x, a),
        }
    }
}
...
```

## License

* [MIT license](./LICENSE.txt)