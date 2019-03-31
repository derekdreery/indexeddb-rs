use futures::{task, Async, Future, Poll};
use std::sync::{Arc, Mutex};

// some helper stuff for the transaction future:
#[derive(Debug)]
pub struct Inner<T, E> {
    completed: bool,
    value: Option<Result<T, E>>,
    task: Option<task::Task>,
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
                task.notify();
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
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut lock = self.inner.lock().unwrap();

        if lock.completed {
            lock.value.take().unwrap().map(Async::Ready)
        } else {
            lock.task = Some(task::current());
            Ok(Async::NotReady)
        }
    }
}
