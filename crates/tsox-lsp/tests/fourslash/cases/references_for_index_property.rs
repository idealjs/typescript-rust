use tsox_lsp::fourslash::Session;


#[test]
fn references_for_index_property() {
    let content = r#"class Foo {
    /*1*/property: number;
    /*2*/method(): void { }
}

var f: Foo;
f["/*3*/property"];
f["/*4*/method"];"#;
    let _s = Session::new_for_test("referencesForIndexProperty", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
