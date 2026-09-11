use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("incrementalUpdateToClassImplementingGenericClass", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_no_errors(&mut s, );
    fourslash::insert(&mut s, "//");
    fourslash::verify_no_errors(&mut s, );
}
