use crate::transaction::{Transaction, TransactionMode};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::object_store::{KeyPath, ObjectStoreDuringUpgrade};

/// A handle on the database during an upgrade.
#[derive(Debug)]
pub struct IdbDatabaseDuringUpgrade {
    inner: web_sys::IdbDatabase,
    request: Arc<web_sys::IdbOpenDbRequest>,
}

impl Deref for IdbDatabaseDuringUpgrade {
    type Target = IdbDatabase;
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.inner) }
    }
}

impl IdbDatabaseDuringUpgrade {
    pub(crate) fn from_raw_unchecked(
        raw: JsValue,
        request: Arc<web_sys::IdbOpenDbRequest>,
    ) -> Self {
        let inner = web_sys::IdbDatabase::unchecked_from_js(raw);
        IdbDatabaseDuringUpgrade { inner, request }
    }

    /// Creates a new object store (roughly equivalent to a table)
    pub fn create_object_store<'a>(
        &'a self,
        name: &str,
        key_path: impl Into<KeyPath>,
        auto_increment: bool,
    ) -> Result<ObjectStoreDuringUpgrade<'a>, JsValue> {
        if self.store_exists(name) {
            return Err(format!("an object store called \"{}\" already exists", name).into());
        }
        let key_path: KeyPath = key_path.into();
        let key_path: JsValue = key_path.into();
        let mut parameters = web_sys::IdbObjectStoreParameters::new();
        parameters.key_path(Some(&key_path));
        parameters.auto_increment(auto_increment);
        let store = self
            .inner
            .create_object_store_with_optional_parameters(name, &parameters)?;
        Ok(ObjectStoreDuringUpgrade {
            inner: store,
            db: self,
        })
    }

    /// Deletes an object store
    pub(crate) fn delete_object_store(&self, name: &str) -> Result<(), JsValue> {
        self.inner.delete_object_store(name)?;
        Ok(())
    }

    /// Is there already a store with the given name?
    fn store_exists(&self, name: &str) -> bool {
        self.object_store_names().iter().any(|test| test == name)
    }

    /// Get the transaction for this request.
    ///
    /// Will panic if called to early.
    pub fn transaction(&self) -> Transaction {
        let inner = self
            .request
            .transaction()
            .expect("transaction not available");
        debug_assert!(inner.mode() == Ok(web_sys::IdbTransactionMode::Versionchange));
        Transaction {
            inner,
            db: PhantomData,
        }
    }
}

/// A handle on the database
#[derive(Debug)]
#[repr(transparent)]
pub struct IdbDatabase {
    pub(crate) inner: web_sys::IdbDatabase,
}

impl IdbDatabase {
    /// The name of the database.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// The current version.
    pub fn version(&self) -> u64 {
        self.inner.version() as u64
    }

    /// Get the names of the object stores in this database.
    pub fn object_store_names(&self) -> Vec<String> {
        to_collection!(self.inner.object_store_names() => Vec<String> : push)
    }

    /// Start a database transaction.
    ///
    /// All operations on data happen within a transaction, including read-only operations.
    pub fn transaction<'a>(&'a self, mode: TransactionMode) -> Transaction<'a> {
        let inner = self
            .inner
            //FIXME(beyer): Transaction should take store_names.
            .transaction_with_str_sequence_and_mode(
                &self.inner.object_store_names().into(),
                mode.into(),
            )
            .unwrap();
        Transaction {
            inner,
            db: PhantomData,
        }
    }
}
