use crate::db::IdbDatabaseDuringUpgrade;
use crate::index::{Index, IndexDuringUpgrade};
use crate::request::IdbRequest;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Debug)]
pub struct ObjectStoreDuringUpgrade<'a> {
    pub(crate) inner: web_sys::IdbObjectStore,
    pub(crate) db: &'a IdbDatabaseDuringUpgrade,
}

impl<'a> ObjectStoreDuringUpgrade<'a> {
    /// Delete this object store.
    pub fn delete(self) -> Result<(), JsValue> {
        self.db.delete_object_store(&self.name())
    }

    pub fn create_index(
        &'a self,
        name: &str,
        key_path: impl Into<KeyPath>,
        unique: bool,
    ) -> Result<IndexDuringUpgrade<'a>, JsValue> {
        let key_path: KeyPath = key_path.into();
        let mut params = web_sys::IdbIndexParameters::new();
        params.unique(unique);
        // https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/createIndex#Exceptions
        // we should be able to check for all error conditions at compile-time,
        // but not yet done.
        let index = self
            .inner
            .create_index_with_str_sequence_and_optional_parameters(
                name,
                &key_path.into(),
                &params,
            )?;
        Ok(IndexDuringUpgrade {
            inner: index,
            parent: self,
        })
    }

    /// Delete an index.
    pub(crate) fn delete_index(&self, name: &str) -> Result<(), JsValue> {
        self.inner.delete_index(name)
    }

    /// Get an already-existing index.
    pub fn index(&'a self, name: &str) -> Result<IndexDuringUpgrade<'a>, JsValue> {
        self.inner
            .index(name)
            .map(|inner| IndexDuringUpgrade::new(inner, self))
    }
}

impl<'a> Deref for ObjectStoreDuringUpgrade<'a> {
    type Target = ObjectStore<'a>;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.inner) }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ObjectStore<'a> {
    pub(crate) inner: web_sys::IdbObjectStore,
    pub(crate) db: PhantomData<&'a ()>,
}

impl<'a> ObjectStore<'a> {
    /// Create a new object store.
    pub(crate) fn new(inner: web_sys::IdbObjectStore) -> Self {
        ObjectStore {
            inner,
            db: PhantomData,
        }
    }

    /// # Properties

    /// Whether they primary key uses an auto-generated incrementing number as
    /// its value.
    pub fn auto_increment(&self) -> bool {
        self.inner.auto_increment()
    }

    /// Get the names of the indexes on this object store.
    pub fn index_names(&self) -> HashSet<String> {
        to_collection!(self.inner.index_names() => HashSet<String> : insert)
    }

    pub fn key_path(&self) -> KeyPath {
        self.inner.key_path().unwrap().into()
    }

    /// The name of the object store.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get an index.
    pub fn index(&'a self, name: &'_ str) -> Result<Index<'a>, JsValue> {
        self.inner.index(name).map(|inner| Index::new(inner, self))
    }

    /// Updates a given record in a database, or inserts a new record if the
    /// given item does not already exist.
    pub fn put<T>(&self, item: T, _key: Option<String>) -> IdbRequest<usize>
    where
        T: serde::ser::Serialize,
    {
        IdbRequest::new(
            self.inner
                .put(&JsValue::from_serde(&item).unwrap())
                .unwrap(),
        )
    }

    pub fn get_all<T>(&self) -> IdbRequest<T> {
        IdbRequest::new(self.inner.get_all().unwrap())
    }
}

/// The path to the key in an object store.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum KeyPath {
    /// Keys are stored *out-of-tree*.
    None,
    /// The path to the single key.
    Single(String),
    /// The paths to all the parts of the key.
    Multi(Vec<String>),
}

impl From<KeyPath> for JsValue {
    fn from(key_path: KeyPath) -> JsValue {
        match key_path {
            KeyPath::None => JsValue::NULL,
            KeyPath::Single(path) => JsValue::from(path),
            KeyPath::Multi(paths) => from_collection!(paths).into(),
        }
    }
}

impl From<JsValue> for KeyPath {
    fn from(val: JsValue) -> Self {
        if val.is_null() || val.is_undefined() {
            KeyPath::None
        } else if let Some(s) = val.as_string() {
            KeyPath::Single(s)
        } else {
            let arr = match val.dyn_into::<js_sys::Array>() {
                Ok(v) => v,
                Err(e) => panic!("expected array of strings, found {:?}", e),
            };
            let mut out = vec![];
            for el in arr.values().into_iter() {
                let el = el.unwrap();
                if let Some(val) = el.as_string() {
                    out.push(val);
                } else {
                    panic!("Expected string, found {:?}", el);
                }
            }
            KeyPath::Multi(out)
        }
    }
}

impl From<Vec<String>> for KeyPath {
    fn from(inner: Vec<String>) -> Self {
        KeyPath::Multi(inner)
    }
}

impl<S> From<&[S]> for KeyPath
where
    S: AsRef<str>,
{
    fn from(inner: &[S]) -> KeyPath {
        KeyPath::Multi(inner.iter().map(|s| s.as_ref().to_owned()).collect())
    }
}

impl From<String> for KeyPath {
    fn from(inner: String) -> KeyPath {
        KeyPath::Single(inner)
    }
}

impl<'a> From<&'a str> for KeyPath {
    fn from(inner: &'a str) -> KeyPath {
        KeyPath::Single(inner.to_owned())
    }
}

impl From<()> for KeyPath {
    fn from((): ()) -> KeyPath {
        KeyPath::None
    }
}
