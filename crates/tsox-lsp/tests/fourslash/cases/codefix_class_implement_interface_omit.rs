use tsox_lsp::fourslash::{self, Session};


#[test]
fn codefix_class_implement_interface_omit() {
    let content = r#"interface One {
    a: number;
    b: string;
}

interface Two extends Omit<One, "a"> {
    c: boolean;
}

class TwoStore implements Two {[| |]}"#;
    let mut s = Session::new_for_test("codefixClassImplementInterface_omit", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
