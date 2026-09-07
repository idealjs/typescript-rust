use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_this4() {
    let content = r#"interface ContextualInterface {
    m: number;
    method(this: this, n: number);
}
let o: ContextualInterface = {
    m: 12,
    method(n) {
        let x = this/*1*/.m;
    }
}
interface ContextualInterface2 {
    (this: void, n: number): void;
}
let contextualInterface2: ContextualInterface2 = function (th/*2*/is, n) { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "this: ContextualInterface", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) this: void", "")
}
