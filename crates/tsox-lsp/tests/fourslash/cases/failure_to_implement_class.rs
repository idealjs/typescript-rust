use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyErrorExistsBetweenMarkers"]
#[test]
fn failure_to_implement_class() {
    let content = r#"interface IExec {
    exec: (filename: string, cmdLine: string) => boolean;
}
class /*1*/NodeExec/*2*/ implements IExec { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
