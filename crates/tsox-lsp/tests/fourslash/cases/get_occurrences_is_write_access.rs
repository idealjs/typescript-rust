use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_is_write_access() {
    let content = r#"var [|{| "isWriteAccess": true |}x|] = 0;
var assignmentRightHandSide = [|{| "isWriteAccess": false |}x|];
var assignmentRightHandSide2 = 1 + [|{| "isWriteAccess": false |}x|];

[|{| "isWriteAccess": true |}x|] = 1;
[|{| "isWriteAccess": true |}x|] = [|{| "isWriteAccess": false |}x|] + [|{| "isWriteAccess": false |}x|];

[|{| "isWriteAccess": false |}x|] == 1;
[|{| "isWriteAccess": false |}x|] <= 1;

var preIncrement = ++[|{| "isWriteAccess": true |}x|];
var postIncrement = [|{| "isWriteAccess": true |}x|]++;
var preDecrement = --[|{| "isWriteAccess": true |}x|];
var postDecrement = [|{| "isWriteAccess": true |}x|]--;

[|{| "isWriteAccess": true |}x|] += 1;
[|{| "isWriteAccess": true |}x|] <<= 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
