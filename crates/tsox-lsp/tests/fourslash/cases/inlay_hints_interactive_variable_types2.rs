use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_variable_types2() {
    let content = r#"const object = { foo: 1, bar: 2 }
const array = [1, 2]
const a = object;
const { foo, bar } = object;
const {} = object;
const b = array;
const [ first, second ] = array;
const [] = array;
declare function foo<T extends number>(t: T): T
const x = foo(1)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
