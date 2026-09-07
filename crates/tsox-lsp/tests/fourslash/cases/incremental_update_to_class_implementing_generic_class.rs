use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn incremental_update_to_class_implementing_generic_class() {
    let content = r#"declare function alert(message?: string): void;
class Animal<T> {
    constructor(public name: T) { }
    move(meters: number) {
        alert(this.name + " moved " + meters + "m.");
    }
}
class Animal2 extends Animal<string> {
    constructor(name: string) { super(name); }
    /*1*/get name2() { return this.name; }
}
var a = new Animal2('eprst');"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::insert(&mut s, "//");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
