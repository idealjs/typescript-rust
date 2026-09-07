use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_object_spread() {
    let content = r#"interface A1 { [|[|{| "contextRangeIndex": 0 |}a|]: number|] };
interface A2 { [|[|{| "contextRangeIndex": 2 |}a|]?: number|] };
let a1: A1;
let a2: A2;
let a12 = { ...a1, ...a2 };
a12.[|a|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[4])
}
