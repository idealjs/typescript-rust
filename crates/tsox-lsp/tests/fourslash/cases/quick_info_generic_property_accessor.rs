use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_generic_property_accessor() {
    let content = r#"
declare const o: {
    f: <T>(x: T) => T
    get g(): <T>(x: T) => T
}

declare const x: number

o.f/*1*/(x)
o.g/*2*/(x)
"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "(property) f: <number>(x: number) => number",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "(accessor) g: <number>(x: number) => number",
        "",
    );
}
