use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) f: <number>(x: number) => number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(accessor) g: <number>(x: number) => number", "")
}
