use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "this: ContextualInterface", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) this: void", "");
}
