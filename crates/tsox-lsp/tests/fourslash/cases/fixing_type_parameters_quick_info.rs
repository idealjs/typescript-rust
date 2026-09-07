use tsox_lsp::fourslash::{self, Session};

#[test]
fn fixing_type_parameters_quick_info() {
    let content = r#"// @strict: false
declare function f<T>(x: T, y: (p: T) => T, z: (p: T) => T): T;
var /*1*/result = /*2*/f(0, /*3*/x => null, /*4*/x => x.blahblah);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var result: number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "function f<number>(x: number, y: (p: number) => number, z: (p: number) => number): number",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) x: number", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) x: number", "");
}
