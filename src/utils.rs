use std::sync::{Arc, Mutex};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

// some helper stuff for the transaction future:
#[derive(Debug)]
pub struct Inner<T, E> {
    completed: bool,
    value: Option<Result<T, E>>,
    task: Option<std::task::Waker>,
}

impl<T, E> Inner<T, E> {
    pub fn new() -> Inner<T, E> {
        Inner {
            completed: false,
            value: None,
            task: None,
        }
    }
}

pub fn transaction_channel<T, E>() -> (TSender<T, E>, TReceiver<T, E>) {
    let inner = Arc::new(Mutex::new(Inner::new()));
    (
        TSender {
            inner: inner.clone(),
        },
        TReceiver { inner },
    )
}

#[derive(Debug, Clone)]
pub struct TSender<T, E> {
    inner: Arc<Mutex<Inner<T, E>>>,
}

impl<T, E> TSender<T, E> {
    pub fn send(&self, value: Result<T, E>) {
        let mut lock = self.inner.lock().unwrap();

        if !lock.completed {
            lock.completed = true;
            lock.value = Some(value);

            if let Some(task) = lock.task.take() {
                drop(lock);
                task.wake_by_ref();
            }
        } else {
            panic!("Only 1 event down channel");
        }
    }
}

#[derive(Debug)]
pub struct TReceiver<T, E> {
    inner: Arc<Mutex<Inner<T, E>>>,
}

impl<T, E> Future for TReceiver<T, E> {
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.inner.lock().unwrap();

        if lock.completed {
            Poll::Ready(lock.value.take().unwrap())
        } else {
            lock.task = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
