use tsox_lsp::fourslash::Session;


#[test]
fn find_references_definition_display_parts() {
    let content = r#"class Gre/*1*/eter {
    someFunction() { th/*2*/is;  }
}

type Options = "opt/*3*/ion 1" | "option 2";
let myOption: Options = "option 1";

some/*4*/Label:
break someLabel;"#;
    let _s = Session::new_for_test("findReferencesDefinitionDisplayParts", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
