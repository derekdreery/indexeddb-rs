use wasm_bindgen::prelude::*;
use futures::Future;
use wasm_bindgen_futures::future_to_promise;
use console_web::println;

#[wasm_bindgen(start)]
pub fn run() {
    spawn_local(indexeddb::open("test", 1).then(|res| {
        match res {
            Ok(ref db) => println!("{:?}", db),
            Err(ref e) => println!("{:?}", e),
        }
        res
    }));
    println!("Hello, world!");
}

pub fn spawn_local(future: impl Future + 'static) {
    future_to_promise(
        future
            .map(|_| JsValue::undefined())
            .map_err(|_| JsValue::undefined()),
    );
}
