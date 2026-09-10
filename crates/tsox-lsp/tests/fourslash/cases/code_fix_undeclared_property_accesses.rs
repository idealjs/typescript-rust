use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_undeclared_property_accesses() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface I { x: number; }
let i: I;
i.y;
i.foo();
enum E { a,b }
let e: typeof E;
e.a;
e.c;
let obj = { a: 1, b: "asdf"};
obj.c;
type T<U> = I | U;
let t: T<number>;
t.x;"#;
    let mut s = Session::new_for_test("codeFixUndeclaredPropertyAccesses", content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, nil)
}
