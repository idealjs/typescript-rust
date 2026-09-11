use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_call_signatures_in_non_generic_types2() {
    let content = r#"interface WrappedArray<T> { }
interface Underscore {
    <T>(list: T[]): WrappedArray<T>;
}
var _: Underscore;
var a: number[];
var /**/b = _(a);  // WrappedArray<any>, should be WrappedArray<number>"#;
    let mut s = Session::new_for_test("genericCallSignaturesInNonGenericTypes2", content);
    fourslash::verify_quick_info_at(&mut s, "", "var b: WrappedArray<number>", "");
}
