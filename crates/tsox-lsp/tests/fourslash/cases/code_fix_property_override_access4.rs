use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_property_override_access4() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const prop = Symbol.for('foo');

class A {
    [prop] = 1;
}
class B extends A {
    get [prop]() { return 2; }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixPropertyOverrideAccessor")
}
