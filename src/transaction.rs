use crate::object_store::ObjectStore;
use std::marker::PhantomData;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DomException, IdbTransactionMode};

#[derive(Debug)]
pub struct Transaction<'a> {
    pub(crate) inner: web_sys::IdbTransaction,
    pub(crate) db: PhantomData<&'a ()>,
}

impl<'a> Transaction<'a> {
    /// # Properties

    /// The error when there is an unsuccessful transaction.
    pub fn error(&self) -> Option<DomException> {
        self.inner.error()
    }

    /// The current mode for accessing the data in the object stores in the
    /// scope of the transaction
    pub fn mode(&self) -> TransactionMode {
        self.inner.mode().unwrap().into()
    }

    /// Get the names of the object stores in this transaction.
    pub fn object_store_names(&self) -> Vec<String> {
        to_collection!(self.inner.object_store_names() => Vec<String> : push)
    }

    /// # Methods

    /// Abort this transaction.
    pub fn abort(&self) -> Result<(), JsValue> {
        self.inner.abort()
    }

    /// Force the transaction to be committed.
    pub fn commit(&self) -> Result<(), JsValue> {
        todo!("Web-sys crate doesn't expose commit");
    }

    /// Get an object store.
    pub fn object_store(&'a self, name: &'_ str) -> Result<ObjectStore<'a>, JsValue> {
        self.inner
            .object_store(name)
            .map(|inner| ObjectStore::new(inner))
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionMode {
    ReadOnly,
    ReadWrite,
    VersionChange,
}

impl From<TransactionMode> for &'static str {
    fn from(m: TransactionMode) -> Self {
        match m {
            TransactionMode::ReadOnly => "readonly",
            TransactionMode::ReadWrite => "readwrite",
            TransactionMode::VersionChange => "versionchange",
        }
    }
}

impl From<TransactionMode> for IdbTransactionMode {
    fn from(m: TransactionMode) -> Self {
        match m {
            TransactionMode::ReadOnly => IdbTransactionMode::Readonly,
            TransactionMode::ReadWrite => IdbTransactionMode::Readwrite,
            TransactionMode::VersionChange => IdbTransactionMode::Versionchange,
        }
    }
}

impl From<IdbTransactionMode> for TransactionMode {
    fn from(m: IdbTransactionMode) -> Self {
        match m {
            IdbTransactionMode::Readonly => TransactionMode::ReadOnly,
            IdbTransactionMode::Readwrite => TransactionMode::ReadWrite,
            IdbTransactionMode::Versionchange => TransactionMode::VersionChange,
            __Nonexhaustive => panic!("Unexpected IdbTransactionMode"),
        }
    }
}
