use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_in_object_literal() {
    let content = r#"interface Foo {
    doStuff(x: string, callback: (a: string) => string);
}
var x1: Foo = {
    y/*1*/1: () => {
        return "";
    } ,
    doStuff: (z, callback) => { return callback(this.y); }
}
var value = 3;
class Foo {
    static getRandomPosition() {
        return {
            "row": v/*2*/alue
        }
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) y1: () => string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var value: number", "")
}
