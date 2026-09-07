use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn qualified_name_import_declaration_with_variable_entity_names() {
    let content = r#"namespace Alpha {
    export var [|{| "name" : "def" |}x|] = 100;
}

namespace Beta {
    import p = Alpha.[|{| "name" : "import" |}x|];
}

var x = Alpha.[|{| "name" : "mem" |}x|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "import");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "import")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "import")
}
