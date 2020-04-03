use std::marker::PhantomData;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

pub struct IdbRequest<'a, T> {
    inner: web_sys::IdbRequest,
    onsuccess: Option<Closure<dyn FnMut()>>,
    onerror: Option<Closure<dyn FnMut()>>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> IdbRequest<'a, T> {
    pub(crate) fn new(inner: web_sys::IdbRequest) -> Self {
        IdbRequest::<T> {
            inner,
            onsuccess: None,
            onerror: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Future for IdbRequest<'a, T>
where
    T: serde::de::DeserializeOwned,
{
    type Output = Result<T, JsValue>;

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
            ReadyState::Done => {
                let result = JsValue::into_serde(&self.inner.result().unwrap()).unwrap();
                Poll::Ready(Ok(result))
            }

            ReadyState::__Nonexhaustive => panic!("unexpected ready state"),
        }
    }
}
