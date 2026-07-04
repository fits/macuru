[english](./README.md)

# Macuru

```Macuru``` はRust用のユーティリティマクロライブラリです。

## ADT (Algebraic data type)

```adt!``` マクロはADT（代数的データ型）の定義を補助するため以下を実施します。

* enum型の生成
    * 要素名は```<要素型>_```
* enum型と要素型の相互変換を実装
* トレイト定義とenum型の実装
    * 関数の戻り値に含まれる```Self```をenum型へ変更

```rust
use macuru::adt;

adt!(
    <enum-type> = <type> | <type> ... [ with <trait-name> {
        <trait-function>;
        ...
    }]
);
```

ただし、次の制限があります。

* 関数（メソッド）のレシーバーは```&self```のみ
* 型やトレイトへのジェネリクス利用は不可

### 例1

```rust
use macuru::adt;

adt!( Data = Elem1 | Elem2 );
```

#### マクロ適用結果

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

### 例2

```rust
use macuru::adt;

adt!( 
    Data = Elem1 | Elem2 with DataFunc {
        fn func1(&self);
        fn func2(&self, a: isize) -> Self;
        fn func3(&self, a: String, b: bool) -> (Self, isize);
        fn func4(&self, a: f32) -> Result<(Self, String, isize), ()>;
    }
);
```

#### マクロ適用結果

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
... 省略
```

## ライセンス

* [MITライセンス](./LICENSE.txt)
