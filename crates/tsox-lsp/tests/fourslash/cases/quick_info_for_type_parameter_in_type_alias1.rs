use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_type_parameter_in_type_alias1() {
    let content = r#"type Ctor<AA> = new () => A/*1*/A;
type MixinCtor<AA> = new () => AA & { constructor: MixinCtor<A/*2*/A> };
type NestedCtor<AA> = new() => AA & (new () => AA & { constructor: NestedCtor<A/*3*/A> });
type Method<AA> = { method(): A/*4*/A };
type Construct<AA> = { new(): A/*5*/A };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(type parameter) AA in type Ctor<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(type parameter) AA in type MixinCtor<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(type parameter) AA in type NestedCtor<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(type parameter) AA in type Method<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(type parameter) AA in type Construct<AA>", "")
}
