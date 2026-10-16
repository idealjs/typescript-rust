use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_union() {
    let content = r#"interface I { x: number; }
interface Many<T> extends ReadonlyArray<T> { extra: number; }
class C { private priv: number; }
const x: I | I[] | Many<string> | C = { /**/ };"#;
    let mut s = Session::new_for_test("completionsUnion", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x"]);
}
