use wasm_bindgen_test::*;

use futures::Future;
use indexeddb::KeyPath;
use wasm_bindgen::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
fn open() -> impl Future<Item = (), Error = JsValue> {
    indexeddb::open("test", 1, |_old_version, _upgrader| ()).map(|_| ())
}

#[wasm_bindgen_test(async)]
fn object_store_params() -> impl Future<Item = (), Error = JsValue> {
    indexeddb::open("test2", 1, |_, upgrader| {
        let obj_store = upgrader
            .create_object_store("test", KeyPath::None, false)
            .unwrap();
        assert_eq!(obj_store.key_path(), KeyPath::None);
        assert_eq!(obj_store.auto_increment(), false);
        drop(obj_store);
        let obj_store = upgrader
            .create_object_store("test2", KeyPath::Single("test".into()), true)
            .unwrap();
        assert_eq!(obj_store.key_path(), KeyPath::Single("test".into()));
        assert_eq!(obj_store.auto_increment(), true);
        drop(obj_store);
        // let obj_store = upgrader
        //     .create_object_store(
        //         "test3",
        //         KeyPath::Multi(vec!["test".into(), "test2".into()]),
        //         false,
        //     )
        //     .unwrap();
        // assert_eq!(
        //     obj_store.key_path(),
        //     KeyPath::Multi(vec!["test".into(), "test2".into()])
        // );
    })
    .map(|_db| ())
}
