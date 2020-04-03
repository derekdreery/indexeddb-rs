#[macro_use]
mod macros;
mod db;
mod error;
mod index;
mod object_store;
mod request;
mod transaction;
mod utils;

pub use crate::db::*;
pub use crate::error::*;
pub use crate::index::*;
pub use crate::object_store::*;
pub use crate::transaction::*;

use std::fmt;
use std::sync::Arc;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::{closure::Closure, JsCast};

#[inline]
fn factory() -> web_sys::IdbFactory {
    web_sys::window().unwrap().indexed_db().unwrap().unwrap()
}

/// Open a database.
pub async fn open(
    name: &str,
    version: u32,
    on_upgrade_needed: impl Fn(u32, IdbDatabaseDuringUpgrade) + 'static,
) -> Result<IdbDatabase> {
    if version == 0 {
        return Err(Error::IdbVersion);
    }
    let mut request = IdbOpenDbRequest::open(name, version).map_err(|_| Error::IdbOpen)?;
    let request_copy = request.inner.clone();
    let onupgradeneeded = move |event: web_sys::IdbVersionChangeEvent| {
        let old_version = cast_version(event.old_version());
        let result = match request_copy.result() {
            Ok(r) => r,
            Err(e) => panic!("Error before ugradeneeded: {:?}", e),
        };
        on_upgrade_needed(
            old_version,
            IdbDatabaseDuringUpgrade::from_raw_unchecked(result, request_copy.clone()),
        );
    };
    let onupgradeneeded =
        Closure::wrap(Box::new(onupgradeneeded) as Box<dyn FnMut(web_sys::IdbVersionChangeEvent)>);
    request
        .inner
        .set_onupgradeneeded(Some(&onupgradeneeded.as_ref().unchecked_ref()));
    request.onupgradeneeded.replace(onupgradeneeded);
    request.await
}

/// Wraps the open db request.
struct IdbOpenDbRequest {
    // We need to move a ref for this into the upgradeneeded closure.
    inner: Arc<web_sys::IdbOpenDbRequest>,
    onsuccess: Option<Closure<dyn FnMut()>>,
    onerror: Option<Closure<dyn FnMut()>>,
    onupgradeneeded: Option<Closure<dyn FnMut(web_sys::IdbVersionChangeEvent)>>,
}

impl IdbOpenDbRequest {
    fn open(name: &str, version: u32) -> Result<IdbOpenDbRequest> {
        // Can error because of origin rules.
        let inner = factory()
            .open_with_f64(name, version as f64)
            .map_err(|_| Error::IdbOpen)?;
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
    type Output = Result<IdbDatabase>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use web_sys::IdbRequestReadyState as ReadyState;

        match self.inner.ready_state() {
            ReadyState::Pending => {
                let success_notifier = cx.waker().clone();
                let error_notifier = cx.waker().clone();

                let onsuccess = Closure::wrap(Box::new(move || {
                    success_notifier.wake_by_ref();
                }) as Box<dyn FnMut()>);
                self.inner
                    .set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
                self.onsuccess.replace(onsuccess); // drop the old closure if there was one

                let onerror = Closure::wrap(Box::new(move || {
                    error_notifier.wake_by_ref();
                }) as Box<dyn FnMut()>);
                self.inner
                    .set_onerror(Some(&onerror.as_ref().unchecked_ref()));
                self.onerror.replace(onerror); // drop the old closure if there was one

                Poll::Pending
            }
            ReadyState::Done => Poll::Ready(Ok(IdbDatabase {
                inner: self.inner.result().unwrap().into(),
            })),
            ReadyState::__Nonexhaustive => panic!("unexpected ready state"),
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
