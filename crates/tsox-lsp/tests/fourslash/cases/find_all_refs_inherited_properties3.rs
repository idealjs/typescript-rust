use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_inherited_properties3() {
    let content = r#"class class1 extends class1 {
    [|/*0*/doStuff() { }|]
    [|/*1*/propName: string;|]
}
interface interface1 extends interface1 {
    [|/*2*/doStuff(): void;|]
    [|/*3*/propName: string;|]
}
class class2 extends class1 implements interface1 {
    [|/*4*/doStuff() { }|]
    [|/*5*/propName: string;|]
}

var v: class2;
v./*6*/doStuff();
v./*7*/propName;"#;
    let mut s = Session::new_for_test("findAllRefsInheritedProperties3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3", "4", "6", "5", "7")
}
