[english](./README.md)

# Macuru

```Macuru``` はRust用のユーティリティマクロライブラリです。

主な目的は、ボイラープレートを排除して開発体験を改善する事です。

## 使い方

```toml
[dependencies]
macuru = { git = "https://github.com/fits/macuru" }
```

```adt!```マクロがenum定義などのボイラープレートコードを生成するので必要な箇所を実装します。

```rust
use macuru::adt;

adt!(
    Data = Empty | NonEmpty derive Clone, Debug with DataFunc {
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

## ADT (Algebraic data type) マクロ

```adt!``` マクロはADT（代数的データ型）の定義を補助するため以下を実施します。

* enum型の生成
    * 要素名は```<要素型>_```
* enum型と要素型の相互変換を実装
* トレイト定義とenum型の実装
    * 関数の戻り値に含まれる```Self```をenum型へ変更
* ジェネリクスの調整
    * enumのジェネリスク定義を要素型で使用するものに限定

```rust
adt!(
    <enum-type> = <type> | <type> ... [ derive <trait>, ... ] [ with <trait-name>
    [ where
        <generic-parameter>: <trait-bounds>
        ...
    ]
    {
        <trait-function>;
        ...
    }]
);
```

ただし、次の注意点があります。

* 関数（メソッド）のレシーバーは```&self```のみ
* ライフタイムとconstをジェネリクス定義に使用できない
* ```x__```を関数（メソッド）の引数名に使用できない

### 目的

RustではADT（sum type）をenumで表現することになりますが、次のような点から保守性等に課題が生じます。

* enumの要素を関数の引数や戻り値として使えない
* enumの要素でトレイトや関数を直接実装できない
* 構造体をenumの要素で包むと、パターンマッチングや変換用のボイラープレートが必要になる

これらを改善してコード品質や開発体験を向上する事が、ADTマクロの主な目的です。

そのため、最も重視したのが以下です。

* コード上に明記した情報だけで実装を完結できる（マクロが生成した暗黙的な型やトレイトを使わせない、意識させない）

マクロが勝手に作った暗黙的な型やトレイトを利用するコーディングスタイルは、コードの可読性を下げ、開発体験を損なうリスクがあり、避けるべきだと考えます。

### 例1

```rust
adt!( Data = Elem1 | Elem2 );
```

#### マクロ適用結果

```rust
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
adt!( Data = Elem1 | Elem2 | Elem3 derive Clone, Debug );
```

#### マクロ適用結果

```rust
#[derive(Clone, Debug)]
pub enum Data {
    Elem1_(Elem1),
    Elem2_(Elem2),
    Elem3_(Elem3),
}

impl From<Elem1> for Data {
    fn from(v: Elem1) -> Self {
        Self::Elem1_(v)
    }
}
...
```

### 例3

```rust
adt!( 
    Data = Elem1 | Elem2 derive Clone, Debug with DataFunc {
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
            Self::Elem1_(x__) => DataFunc::func1(x__),
            Self::Elem2_(x__) => DataFunc::func1(x__),
        }
    }

    fn func2(&self, a: isize) -> Data {
        match self {
            Self::Elem1_(x__) => DataFunc::func2(x__, a),
            Self::Elem2_(x__) => DataFunc::func2(x__, a),
        }
    }

    fn func3(&self, a: String, b: bool) -> (Data, isize) {
        match self {
            Self::Elem1_(x__) => DataFunc::func3(x__, a, b),
            Self::Elem2_(x__) => DataFunc::func3(x__, a, b),
        }
    }

    fn func4(&self, a: f32) -> Result<(Data, String, isize), ()> {
        match self {
            Self::Elem1_(x__) => DataFunc::func4(x__, a),
            Self::Elem2_(x__) => DataFunc::func4(x__, a),
        }
    }
}
...
```

### 例4

```rust
adt!( 
    Data<A, B> = Elem1<A> | Elem2<A, B> derive Debug with DataFunc<A, B>
    where
        A: Clone,
        B: Copy + Default + PartialOrd + Add<Output = B>,
    {
        fn id(&self) -> A;
        fn value(&self) -> B;
        fn add(&self, v: B) -> Option<Self>;
    }
);
```

#### マクロ適用結果

```rust
#[derive(Debug)]
pub enum Data<A, B> {
    Elem1_(Elem1<A>),
    Elem2_(Elem2<A, B>),
}

pub trait DataFunc<A, B> {
    fn id(&self) -> A;
    fn value(&self) -> B;
    fn add(&self, v: B) -> Option< Data<A, B> >;
}

impl<A, B> DataFunc<A, B> for Data<A, B>
where
    A: Clone,
    B: Copy + Default + PartialOrd + Add<Output = B>,
{
    fn id(&self) -> A {
        match self {
            Self::Elem1_(x__) => DataFunc::<A, B>::id(x__),
            Self::Elem2_(x__) => DataFunc::<A, B>::id(x__),
        }
    }

    fn value(&self) -> B {
        match self {
            Self::Elem1_(x__) => DataFunc::<A, B>::value(x__),
            Self::Elem2_(x__) => DataFunc::<A, B>::value(x__),
        }
    }

    fn add(&self, v: B) -> Option< Data<A, B> > {
        match self {
            Self::Elem1_(x__) => DataFunc::<A, B>::add(x__, v),
            Self::Elem2_(x__) => DataFunc::<A, B>::add(x__, v),
        }
    }
}

impl<A, B> From< Elem1<A> > for Data<A, B> {
    fn from(v: Elem1<A>) -> Self {
        Self::Elem1_(v)
    }
}

impl<A, B> TryFrom< Data<A, B> > for Elem1<A> {
    type Error = ();

    fn try_from(v: Data<A, B>) -> Result<Self, Self::Error> {
        if let Data::Elem1_(x) = v {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl<A, B> From< Elem2<A, B> > for Data<A, B> {
    fn from(v: Elem2<A, B>) -> Self {
        Self::Elem2_(v)
    }
}

impl<A, B> TryFrom< Data<A, B> > for Elem2<A, B> {
    type Error = ();

    fn try_from(v: Data<A, B>) -> Result<Self, Self::Error> {
        if let Data::Elem2_(x) = v {
            Ok(x)
        } else {
            Err(())
        }
    }
}
```

### 例5

```rust
adt!(
    Data<A> = Elem1 | Elem2 with DataFunc<A> {
        fn func1(&self) -> A;
    }
);
```

#### マクロ適用結果

```rust
pub enum Data {
    Elem1_(Elem1),
    Elem2_(Elem2),
}

pub trait DataFunc<A> {
    fn func1(&self) -> A;
}

impl<A> DataFunc<A> for Data {
    fn func1(&self) -> A {
        match self {
            Self::Elem1_(x__) => DataFunc::<A>::func1(x__),
            Self::Elem2_(x__) => DataFunc::<A>::func1(x__),
        }
    }
}
...
```

## ライセンス

* [MITライセンス](./LICENSE.txt)
