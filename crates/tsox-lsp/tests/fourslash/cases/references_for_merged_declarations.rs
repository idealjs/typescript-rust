use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_merged_declarations() {
    let content = r#"/*1*/interface /*2*/Foo {
}

/*3*/module /*4*/Foo {
    export interface Bar { }
}

/*5*/function /*6*/Foo(): void {
}

var f1: /*7*/Foo.Bar;
var f2: /*8*/Foo;
/*9*/Foo.bind(this);"#;
    let mut s = Session::new_for_test("referencesForMergedDeclarations", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9")
}
