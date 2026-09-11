use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_no_truncation_properties() {
    let content = r#"type props = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z";
type manyprops = `${props}${props}`;

interface Foo<T extends string> {
    manyProps(a: {[K in T]: {[K2 in T]: `${K}.${K2}`}}): void;
}
    
class Bar implements Foo<manyprops> {
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceNoTruncationProperties", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
