use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_default_export02() {
    let content = r#"/*1*/export default function /*2*/DefaultExportedFunction() {
    return /*3*/DefaultExportedFunction;
}

var x: typeof /*4*/DefaultExportedFunction;

var y = /*5*/DefaultExportedFunction();

/*6*/namespace /*7*/DefaultExportedFunction {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7")
}
