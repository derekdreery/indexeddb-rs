#[macro_use]
mod macros;
mod db;
mod index;
mod object_store;
mod utils;
mod transaction;

pub use crate::db::*;
pub use crate::index::*;
pub use crate::object_store::*;
pub use crate::transaction::*;
use futures::{
    future::{self, Either},
    task, Async, Future, Poll,
};
use std::fmt;
use std::sync::Arc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

#[inline]
fn factory() -> web_sys::IdbFactory {
    web_sys::window().unwrap().indexed_db().unwrap().unwrap()
}

//const MAX_SAFE_INTEGER: u64 = 9007199254740991; // 2 ^ 53

/// Open a database.
///
/// # Panics
///
/// This function will panic if the new version is 0.
pub fn open(
    name: &str,
    version: u32,
    on_upgrade_needed: impl Fn(u32, DbDuringUpgrade) + 'static,
) -> impl Future<Item = Db, Error = JsValue> {
    if version == 0 {
        panic!("indexeddb version must be >= 1");
    }
    let mut request = match IdbOpenDbRequest::open(name, version) {
        Ok(request) => request,
        Err(e) => return Either::B(future::err(e)),
    };
    let request_copy = request.inner.clone();
    let onupgradeneeded = move |event: web_sys::IdbVersionChangeEvent| {
        let old_version = cast_version(event.old_version());
        let result = match request_copy.result() {
            Ok(r) => r,
            Err(e) => panic!("Error before ugradeneeded: {:?}", e),
        };
        on_upgrade_needed(
            old_version,
            DbDuringUpgrade::from_raw_unchecked(result, request_copy.clone()),
        );
    };
    let onupgradeneeded =
        Closure::wrap(Box::new(onupgradeneeded) as Box<dyn FnMut(web_sys::IdbVersionChangeEvent)>);
    request
        .inner
        .set_onupgradeneeded(Some(&onupgradeneeded.as_ref().unchecked_ref()));
    request.onupgradeneeded.replace(onupgradeneeded);
    Either::A(request)
}

/// Wraps the open db request. Private - the user interacts with the request using the function
/// passed to the `open` method.
struct IdbOpenDbRequest {
    // We need to move a ref for this into the upgradeneeded closure.
    inner: Arc<web_sys::IdbOpenDbRequest>,
    onsuccess: Option<Closure<dyn FnMut()>>,
    onerror: Option<Closure<dyn FnMut()>>,
    onupgradeneeded: Option<Closure<dyn FnMut(web_sys::IdbVersionChangeEvent)>>,
}

impl IdbOpenDbRequest {
    fn open(name: &str, version: u32) -> Result<IdbOpenDbRequest, JsValue> {
        // Can error because of origin rules.
        let inner = factory().open_with_f64(name, version as f64)?;
        Ok(IdbOpenDbRequest {
            inner: Arc::new(inner),
            onsuccess: None,
            onerror: None,
            onupgradeneeded: None,
        })
    }
}

impl fmt::Debug for IdbOpenDbRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdbOpenDbRequest")
    }
}

impl Future for IdbOpenDbRequest {
    type Item = Db;
    type Error = JsValue;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use web_sys::IdbRequestReadyState as ReadyState;
        match self.inner.ready_state() {
            ReadyState::Pending => {
                let success_notifier = task::current();
                let error_notifier = success_notifier.clone();
                // If we're not ready set up onsuccess and onerror callbacks to notify the
                // executor.
                let onsuccess = Closure::wrap(Box::new(move || {
                    success_notifier.notify();
                }) as Box<FnMut()>);
                self.inner
                    .set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
                self.onsuccess.replace(onsuccess); // drop the old closure if there was one

                let onerror = Closure::wrap(Box::new(move || {
                    error_notifier.notify();
                }) as Box<FnMut()>);
                self.inner
                    .set_onerror(Some(&onerror.as_ref().unchecked_ref()));
                self.onerror.replace(onerror); // drop the old closure if there was one

                Ok(Async::NotReady)
            }
            ReadyState::Done => match self.inner.result() {
                Ok(val) => Ok(Async::Ready(Db {
                    inner: val.unchecked_into(),
                })),
                Err(_) => match self.inner.error() {
                    Ok(Some(e)) => Err(e.into()),
                    Ok(None) => unreachable!("internal error polling open db request"),
                    Err(e) => Err(e),
                },
            },
            _ => panic!("unexpected ready state"),
        }
    }
}

// Some u64 numbers cannot be represented as f64. This checks as part of the cast.
// https://stackoverflow.com/questions/3793838/which-is-the-first-integer-that-an-ieee-754-float-is-incapable-of-representing-e
fn cast_version(val: f64) -> u32 {
    if val < 0.0 || val > u32::max_value() as f64 {
        panic!("out of bounds");
    }
    val as u32
}

#[test]
fn test_cast() {
    for val in vec![0u32, 1, 10] {
        assert_eq!(cast_version(val as f64), val);
    }
}

#[test]
#[should_panic]
fn test_cast_too_big() {
    cast_version((1u64 << 54) as f64);
}
