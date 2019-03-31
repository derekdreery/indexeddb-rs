use console_web::println;
use futures::Future;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let version = 1;
    spawn_local(
        indexeddb::open("test", version, move |old_version, db| {
            if version >= 1 {
                let store = db.create_object_store("contact", "id", true).unwrap();
                store
                    .create_index("idx_given_name", "given_name", false)
                    .unwrap();
                store
                    .create_index("idx_family_name", "family_name", false)
                    .unwrap();
            }
        })
        .then(|res| {
            match res {
                Ok(ref db) => println!("Success: {:?}", db),
                Err(ref e) => println!("Error: {:?}", e),
            }
            res
        }),
    );
}

pub fn spawn_local(future: impl Future + 'static) {
    wasm_bindgen_futures::spawn_local(future.map(|_| ()).map_err(|_| ()));
}
