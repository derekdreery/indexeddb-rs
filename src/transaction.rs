use std::marker::PhantomData;
use web_sys::IdbTransactionMode;


#[derive(Debug)]
pub struct Transaction<'a> {
    pub(crate) inner: web_sys::IdbTransaction,
    pub(crate) db: PhantomData<&'a ()>,
}

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
            TransactionMode::VersionChange=> "versionchange",
        }
    }
}

impl From<TransactionMode> for IdbTransactionMode {
    fn from(m: TransactionMode) -> Self {
        match m {
            TransactionMode::ReadOnly => IdbTransactionMode::Readonly,
            TransactionMode::ReadWrite => IdbTransactionMode::Readwrite,
            TransactionMode::VersionChange=> IdbTransactionMode::Versionchange,
        }
    }
}