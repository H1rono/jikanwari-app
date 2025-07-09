use serde::{Deserialize, Serialize};

/// [newtype](https://doc.rust-jp.rs/rust-by-example-ja/generics/new_types.html) に実装を追加するマクロです。
///
/// # Examples
///
/// ```ignore
/// # use crate::newtype;
/// newtype! {
///     #[derive(Debug, Clone, Copy)]
///     pub struct MyI32(pub(crate) i32);
/// }
/// ```
///
/// これは以下のように展開されます。
///
/// ```
/// #[derive(Debug, Clone, Copy)]
/// pub struct MyI32(pub(crate) i32);
///
/// impl MyI32 {
///     pub fn new(inner: i32) -> Self { Self(inner) }
///     pub fn as_inner(&self) -> &i32 { &self.0 }
///     pub fn mut_inner(&mut self) -> &mut i32 { &mut self.0 }
///     pub fn into_inner(self) -> i32 { self.0 }
/// }
///
/// impl std::convert::AsRef<i32> for MyI32 {
///     fn as_ref(&self) -> &i32 { &self.0 }
/// }
///
/// impl std::convert::AsMut<i32> for MyI32 {
///     fn as_mut(&mut self) -> &mut i32 { &mut self.0 }
/// }
///
/// impl std::convert::From<i32> for MyI32 {
///     fn from(value: i32) -> Self { Self::new(value) }
/// }
///
/// impl std::convert::From<MyI32> for i32 {
///     fn from(value: MyI32) -> Self { value.into_inner() }
/// }
/// ```
///
/// 定義する struct では任意の [visibility] および [attributes] を指定できます。
///
/// ```ignore
/// # use serde::{Deserialize, Serialize};
/// # use crate::newtype;
/// newtype! {
///     /// some doc comments
///     #[derive(Deserialize, Serialize)]
///     #[serde(transparent)]
///     pub(crate) struct Name(String);
/// }
/// ```
///
/// [visibility]: https://doc.rust-lang.org/reference/visibility-and-privacy.html
/// [attributes]: https://doc.rust-lang.org/reference/attributes.html
macro_rules! newtype {
    {
        $(#[$meta_container:meta])*
        $vis_struct:vis struct $newtype:ident(
            $(#[$meta_inner:meta])*
            $vis_inner:vis $inner:ty
        );
    } => {
        $(#[$meta_container])*
        $vis_struct struct $newtype(
            $(#[$meta_inner])*
            $vis_inner $inner
        );

        impl $newtype {
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }

            #[must_use]
            pub fn as_inner(&self) -> & $inner {
                &self.0
            }

            #[must_use]
            pub fn mut_inner(&mut self) -> &mut $inner {
                &mut self.0
            }

            #[must_use]
            pub fn into_inner(self) -> $inner {
                self.0
            }
        }

        impl ::std::convert::AsRef<$inner> for $newtype {
            fn as_ref(&self) -> & $inner {
                self.as_inner()
            }
        }

        impl ::std::convert::AsMut<$inner> for $newtype {
            fn as_mut(&mut self) -> &mut $inner {
                self.mut_inner()
            }
        }

        impl ::std::convert::From<$inner> for $newtype {
            fn from(value: $inner) -> Self {
                Self::new(value)
            }
        }

        impl ::std::convert::From<$newtype> for $inner {
            fn from(value: $newtype) -> Self {
                value.into_inner()
            }
        }
    };
}

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub trait Error: std::error::Error + Send + Sync + 'static {}

impl<E> Error for E where E: std::error::Error + Send + Sync + 'static {}

newtype! {
    #[must_use]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
    pub struct UserId(uuid::Uuid);
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

newtype! {
    #[must_use]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
    pub struct GroupId(uuid::Uuid);
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub members: Vec<UserId>,
}
