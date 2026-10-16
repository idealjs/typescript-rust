use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern13() {
    let content = r#"interface I {
    x: number;
    y: string;
    z: boolean;
}

interface J {
    x: string;
    y: string;
}

let { /**/ }: I | J = { x: 10 };"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern13", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x", "y"]);
}
