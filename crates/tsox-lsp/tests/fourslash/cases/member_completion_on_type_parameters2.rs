use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_completion_on_type_parameters2() {
    let content = r#"class A {
    foo(): string { return ''; }
}

class B extends A {
    bar(): string {
        return '';
    }
}

class C<U extends A, T extends A> {
    x: U;
    y = this.x./**/ // completion list here
}"#;
    let mut s = Session::new_for_test("memberCompletionOnTypeParameters2", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo"]);
}
