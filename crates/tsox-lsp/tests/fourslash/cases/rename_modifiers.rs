use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_modifiers() {
    let content = r#"[|[|declare|] [|abstract|] class [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -3 |}C1|] {
    [|[|static|] [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}a|];|]
    [|[|readonly|] [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}b|];|]
    [|[|public|] [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}c|];|]
    [|[|protected|] [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}d|];|]
    [|[|private|] [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}e|];|]
}|]
[|[|const|] enum [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}E|] {
}|]
[|[|async|] function [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -2 |}fn|]() {}|]
[|[|export|] [|default|] class [|{| "isWriteAccess": true, "isDefinition": true, "contextRangeDelta": -3 |}C2|] {}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[2], f.Ranges()[5], f.Ranges
}
