/*

pub(crate) trait FromJs<From> {
    fn from_js(f: From) -> Result<Self, JsValue>;
}

pub(crate) trait IntoJs<Into> {
    fn into_js(self) -> Result<Into, JsValue>
}

fn vec_to_dom_string_array(list: Vec<String>) -> js_sys::Array {
    let arr = js_sys::Array::new();
    for path in list {
        arr.push(&JsValue::from(path));
    }
    arr
}
*/

macro_rules! to_collection {
    ($js:expr => $coll:tt<$inner:ty> : $method:tt) => {{
        let input = $js;
        let mut list = $coll::new();
        for i in 0..input.length() {
            list.$method(input.get(i).unwrap());
        }
        list
    }};
}

macro_rules! from_collection {
    ($coll:expr) => {{
        let coll = $coll;
        let arr = js_sys::Array::new();
        for el in coll.iter() {
            arr.push(&el.into());
        }
        arr
    }};
}
