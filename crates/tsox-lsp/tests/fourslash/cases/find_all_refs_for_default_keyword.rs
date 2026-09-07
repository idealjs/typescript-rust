use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_default_keyword() {
    let content = r#"// @noLib: true
function f(value: string, /*1*/default: string) {}

const /*2*/default = 1;

function /*3*/default() {}

class /*4*/default {}

const foo = {
    /*5*/default: 1
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
