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
