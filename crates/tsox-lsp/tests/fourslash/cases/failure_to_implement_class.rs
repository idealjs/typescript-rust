use tsox_lsp::fourslash::{self, Session};


#[test]
fn failure_to_implement_class() {
    let content = r#"interface IExec {
    exec: (filename: string, cmdLine: string) => boolean;
}
class /*1*/NodeExec/*2*/ implements IExec { }"#;
    let mut s = Session::new_for_test("failureToImplementClass", content);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
