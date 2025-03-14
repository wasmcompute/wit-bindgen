wit_bindgen_guest_rust::generate!({
    import: "../../tests/runtime/lists/imports.wit",
    default: "../../tests/runtime/lists/exports.wit",
    name: "exports",
});

struct Component;

export_exports!(Component);

impl exports::Exports for Component {
    fn allocated_bytes() -> u32 {
        test_rust_wasm::get() as u32
    }

    fn test_imports() {
        use imports::*;

        let _guard = test_rust_wasm::guard();

        empty_list_param(&[]);
        empty_string_param("");
        assert!(empty_list_result().is_empty());
        assert!(empty_string_result().is_empty());

        list_param(&[1, 2, 3, 4]);
        list_param2("foo");
        list_param3(&["foo", "bar", "baz"]);
        list_param4(&[&["foo", "bar"], &["baz"]]);
        assert_eq!(list_result(), [1, 2, 3, 4, 5]);
        assert_eq!(list_result2(), "hello!");
        assert_eq!(list_result3(), ["hello,", "world!"]);

        assert_eq!(list_roundtrip(&[]), []);
        assert_eq!(list_roundtrip(b"x"), b"x");
        assert_eq!(list_roundtrip(b"hello"), b"hello");

        assert_eq!(string_roundtrip("x"), "x");
        assert_eq!(string_roundtrip(""), "");
        assert_eq!(string_roundtrip("hello"), "hello");
        assert_eq!(string_roundtrip("hello ⚑ world"), "hello ⚑ world");

        assert_eq!(
            list_minmax8(&[u8::MIN, u8::MAX], &[i8::MIN, i8::MAX]),
            (vec![u8::MIN, u8::MAX], vec![i8::MIN, i8::MAX]),
        );
        assert_eq!(
            list_minmax16(&[u16::MIN, u16::MAX], &[i16::MIN, i16::MAX]),
            (vec![u16::MIN, u16::MAX], vec![i16::MIN, i16::MAX]),
        );
        assert_eq!(
            list_minmax32(&[u32::MIN, u32::MAX], &[i32::MIN, i32::MAX]),
            (vec![u32::MIN, u32::MAX], vec![i32::MIN, i32::MAX]),
        );
        assert_eq!(
            list_minmax64(&[u64::MIN, u64::MAX], &[i64::MIN, i64::MAX]),
            (vec![u64::MIN, u64::MAX], vec![i64::MIN, i64::MAX]),
        );
        assert_eq!(
            list_minmax_float(
                &[f32::MIN, f32::MAX, f32::NEG_INFINITY, f32::INFINITY],
                &[f64::MIN, f64::MAX, f64::NEG_INFINITY, f64::INFINITY]
            ),
            (
                vec![f32::MIN, f32::MAX, f32::NEG_INFINITY, f32::INFINITY],
                vec![f64::MIN, f64::MAX, f64::NEG_INFINITY, f64::INFINITY],
            ),
        );
    }

    fn empty_list_param(a: Vec<u8>) {
        assert!(a.is_empty());
    }

    fn empty_string_param(a: String) {
        assert!(a.is_empty());
    }

    fn empty_list_result() -> Vec<u8> {
        Vec::new()
    }

    fn empty_string_result() -> String {
        String::new()
    }

    fn list_param(list: Vec<u8>) {
        assert_eq!(list, [1, 2, 3, 4]);
    }

    fn list_param2(ptr: String) {
        assert_eq!(ptr, "foo");
    }

    fn list_param3(ptr: Vec<String>) {
        assert_eq!(ptr.len(), 3);
        assert_eq!(ptr[0], "foo");
        assert_eq!(ptr[1], "bar");
        assert_eq!(ptr[2], "baz");
    }

    fn list_param4(ptr: Vec<Vec<String>>) {
        assert_eq!(ptr.len(), 2);
        assert_eq!(ptr[0][0], "foo");
        assert_eq!(ptr[0][1], "bar");
        assert_eq!(ptr[1][0], "baz");
    }

    fn list_result() -> Vec<u8> {
        vec![1, 2, 3, 4, 5]
    }

    fn list_result2() -> String {
        "hello!".to_string()
    }

    fn list_result3() -> Vec<String> {
        vec!["hello,".to_string(), "world!".to_string()]
    }

    fn list_roundtrip(x: Vec<u8>) -> Vec<u8> {
        x.clone()
    }

    fn string_roundtrip(x: String) -> String {
        x.clone()
    }
}
