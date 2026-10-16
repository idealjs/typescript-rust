use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_merged_interfaces() {
    let content = r#"namespace M {
    interface A<T> {
        (): string;
        (x: T): T;
    }
    interface A<T> {
        (x: T, y: number): T;
        <U>(x: U, y: T): U;
    }
    var a: A<boolean>;
    var r = a();
    var r2 = a(true);
    var r3 = a(true, 2);
    var /*1*/r4 = a(1, true);
}"#;
    let mut s = Session::new_for_test("quickInfoOnMergedInterfaces", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r4: number", "");
}
