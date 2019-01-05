
use futures::{future, Future, Poll, Async, task};
use wasm_bindgen::{JsValue, JsCast, closure::Closure};

#[inline]
fn factory() -> web_sys::IdbFactory {
     web_sys::window().unwrap().indexed_db().unwrap().unwrap()
}

#[derive(Debug)]
pub struct Db {
    inner: web_sys::IdbDatabase,
}

pub fn open(name: &str, version: u32) -> impl Future<Item=Db, Error=JsValue> {
    if version == 0 {
        panic!("indexeddb version must be >= 1");
    }
    // todo can we avoid dynamic dispatch - and is it any better?
    match IdbOpenDbRequest::open(name, version) {
        Ok(f) => Box::new(f) as Box<dyn Future<Item=Db, Error=JsValue>>,
        Err(e) => Box::new(future::err(e)) as Box<dyn Future<Item=Db, Error=JsValue>>
    }
}

struct IdbOpenDbRequest {
    inner: web_sys::IdbOpenDbRequest,
    onsuccess: Option<Closure<FnMut()>>,
    onerror: Option<Closure<FnMut()>>,
}

impl IdbOpenDbRequest {
    fn open(name: &str, version: u32) -> Result<IdbOpenDbRequest, JsValue> {
        // Can error because of origin rules.
        let inner = factory().open_with_u32(name, version)?;
        Ok(IdbOpenDbRequest {
            inner,
            onsuccess: None,
            onerror: None,
        })
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
                let onsuccess = Closure::wrap(Box::new(move || {
                    success_notifier.notify();
                }) as Box<FnMut()>);
                self.inner.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
                self.onsuccess.replace(onsuccess); // drop the old closure if there was one

                let onerror = Closure::wrap(Box::new(move || {
                    error_notifier.notify();
                }) as Box<FnMut()>);
                self.inner.set_onerror(Some(&onerror.as_ref().unchecked_ref()));
                self.onerror.replace(onerror); // drop the old closure if there was one

                Ok(Async::NotReady)
            },
            ReadyState::Done => {
                match self.inner.result() {
                    Ok(val) => Ok(Async::Ready(Db { inner: val.unchecked_into() })),
                    Err(_) => match self.inner.error() {
                        Ok(Some(e)) => Err(e.into()),
                        Ok(None) => unreachable!("internal error polling open db request"),
                        Err(e) => Err(e)
                    }
                }
            },
            _ => panic!("unexpected ready state")
        }
    }
}
