use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_discriminated_union() {
    let content = r#"interface A { kind: "a"; a: number; }
interface B { kind: "b"; b: number; }
const c: A | B = { kind: "a", /**/ };"#;
    let mut s = Session::new_for_test("completionsDiscriminatedUnion", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["a"]);
}
