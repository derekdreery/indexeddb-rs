use crate::object_store::{ObjectStore, ObjectStoreDuringUpgrade};
use crate::request::IdbRequest;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use wasm_bindgen::prelude::*;

/// An index during a database upgrade
#[derive(Debug)]
pub struct IndexDuringUpgrade<'a> {
    pub(crate) inner: web_sys::IdbIndex,
    pub(crate) parent: &'a ObjectStoreDuringUpgrade<'a>,
}

impl<'a> Deref for IndexDuringUpgrade<'a> {
    type Target = Index<'a>;
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.inner) }
    }
}

impl<'a> IndexDuringUpgrade<'a> {
    pub(crate) fn new(inner: web_sys::IdbIndex, parent: &'a ObjectStoreDuringUpgrade<'a>) -> Self {
        IndexDuringUpgrade { inner, parent }
    }

    /// Deletes the index.
    pub fn delete(self) -> Result<(), JsValue> {
        self.parent.delete_index(&self.name())
    }
}

/// An index
#[repr(transparent)]
#[derive(Debug)]
pub struct Index<'a> {
    inner: web_sys::IdbIndex,
    parent: PhantomData<&'a ()>,
}

impl<'a> Index<'a> {
    pub(crate) fn new(inner: web_sys::IdbIndex, _: &'a ObjectStore<'a>) -> Self {
        Index {
            inner,
            parent: PhantomData,
        }
    }

    pub fn count(&self) -> IdbRequest {
        IdbRequest {
            inner: self.inner.count().unwrap(),
            onsuccess: None,
            onerror: None,
        }
    }

    pub fn name(&self) -> String {
        self.inner.name()
    }

    pub fn get<T>(&self, key: T) -> IdbRequest
    where
        T: serde::ser::Serialize,
    {
        IdbRequest {
            inner: self.inner.get(&JsValue::from_serde(&key).unwrap()).unwrap(),
            onsuccess: None,
            onerror: None,
        }
    }

    pub fn get_all_keys(&self) -> IdbRequest {
        IdbRequest {
            inner: self.inner.get_all_keys().unwrap(),
            onerror: None,
            onsuccess: None,
        }
    }
}
