use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: for _, marker := range markers {"]
#[test]
fn rename_default_keyword() {
    let content = r#"
// @noLib: true
function f(value: string, /*1*/default: string) {}

const /*2*/default = 1;

function /*3*/default() {}

class /*4*/default {}

const foo = {
    /*5*/[|default|]: 1
}"#;
    let mut s = Session::new(content);
    // TODO: markers := []string{"1", "2", "3", "4"}
    // TODO: for _, marker := range markers {
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
