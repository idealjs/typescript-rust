use tsox_lsp::fourslash::{self, Session};

#[test]
fn member_overload_edits() {
    let content = r#"namespace M {
    export class A {
        public m(n: number) {
            return 0;
        }
        public n() {
            return this.m(0);
        }
    }
    export class B extends A { /*1*/ }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "public m(n: number) { return 0; }");
    fourslash::verify_no_errors(&mut s);
}
