use indexeddb::{KeyPath, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestAccount {
    uuid: String,
    name: String,
    domain: String,
    user: TestUser,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestUser {
    uuid: String,
    name: String,
    email: String,
}

#[wasm_bindgen_test(async)]
async fn open() {
    assert!(
        indexeddb::open("open_test_db", 1, |_old_version, _upgrader| ())
            .await
            .is_ok()
    );
}

#[wasm_bindgen_test(async)]
async fn object_store_params() {
    assert!(indexeddb::open("object_store_test_db", 1, |_, upgrader| {
        let obj_store = upgrader
            .create_object_store("object_store", KeyPath::None, false)
            .unwrap();
        assert_eq!(obj_store.key_path(), KeyPath::None);
        assert_eq!(obj_store.auto_increment(), false);
        drop(obj_store);
        let obj_store = upgrader
            .create_object_store(
                "object_store_test_db_2",
                KeyPath::Single("key_path_single".into()),
                true,
            )
            .unwrap();
        assert_eq!(
            obj_store.key_path(),
            KeyPath::Single("key_path_single".into())
        );
        assert_eq!(obj_store.auto_increment(), true);
        drop(obj_store);
        let obj_store = upgrader
            .create_object_store(
                "key_path_multi",
                KeyPath::Multi(vec!["test".into(), "test2".into()]),
                false,
            )
            .unwrap();
        assert_eq!(
            obj_store.key_path(),
            KeyPath::Multi(vec!["test".into(), "test2".into()])
        );
    })
    .await
    .is_ok());
}

#[wasm_bindgen_test(async)]
async fn object_store_and_index() {
    let db = indexeddb::open("object_store_index_test_db", 1, |_, upgrader| {
        let obj_store = upgrader
            .create_object_store("accounts", KeyPath::Single("id".into()), true)
            .unwrap();
        obj_store.create_index("a_by_uuid", "uuid", true).unwrap();
    })
    .await
    .unwrap();

    assert_eq!(db.object_store_names(), vec!["accounts".to_string()]);
    let tx = db.transaction(TransactionMode::ReadWrite);
    assert_eq!(tx.object_store_names(), vec!["accounts".to_string()]);
    assert_eq!(tx.mode(), TransactionMode::ReadWrite);
    let object_store = tx.object_store("accounts").unwrap();

    object_store
        .put(
            TestAccount {
                uuid: "I6WABHBQEWIMDMWWBRHDCIAXGQ".to_string(),
                name: "AgileBits".to_string(),
                domain: "agilebits.1password.com".to_string(),
                user: TestUser {
                    uuid: "STBEASXUNJKRKXDQ3URUN667UQ".to_string(),
                    name: "Wendy Appleseed".to_string(),
                    email: "wendy@appleseed.me".to_string(),
                },
            },
            None,
        )
        .await
        .unwrap();
    object_store
        .put(
            TestAccount {
                uuid: "B4PSCIHJKZDMZOYJR5P7LRPSUA".to_string(),
                name: "Wendy Appleseed".to_string(),
                domain: "my.1password.com".to_string(),
                user: TestUser {
                    uuid: "J7ZTAVR5VTXXWMLAYZR2ZMFS7A".to_string(),
                    name: "Wendy Appleseed".to_string(),
                    email: "wendy@appleseed.me".to_string(),
                },
            },
            None,
        )
        .await
        .unwrap();

    let index = object_store.index("a_by_uuid").unwrap();
    assert_eq!(index.name(), "a_by_uuid");

    let keys = index.get_all_keys().await.unwrap();
    assert_eq!(keys, vec![2, 1]);

    let count = index.count().await.unwrap();
    assert_eq!(count, 2);

    let test: Vec<TestAccount> = object_store.get_all().await.unwrap();
    assert_eq!(
        test,
        vec![
            TestAccount {
                uuid: "I6WABHBQEWIMDMWWBRHDCIAXGQ".to_string(),
                name: "AgileBits".to_string(),
                domain: "agilebits.1password.com".to_string(),
                user: TestUser {
                    uuid: "STBEASXUNJKRKXDQ3URUN667UQ".to_string(),
                    name: "Wendy Appleseed".to_string(),
                    email: "wendy@appleseed.me".to_string(),
                },
            },
            TestAccount {
                uuid: "B4PSCIHJKZDMZOYJR5P7LRPSUA".to_string(),
                name: "Wendy Appleseed".to_string(),
                domain: "my.1password.com".to_string(),
                user: TestUser {
                    uuid: "J7ZTAVR5VTXXWMLAYZR2ZMFS7A".to_string(),
                    name: "Wendy Appleseed".to_string(),
                    email: "wendy@appleseed.me".to_string(),
                },
            },
        ]
    )
}
