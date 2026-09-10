use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_index_property() {
    let content = r#"class Foo {
    /*1*/property: number;
    /*2*/method(): void { }
}

var f: Foo;
f["/*3*/property"];
f["/*4*/method"];"#;
    let mut s = Session::new_for_test("referencesForIndexProperty", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
