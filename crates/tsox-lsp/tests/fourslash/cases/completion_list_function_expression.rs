use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.InsertLine"]
#[test]
fn completion_list_function_expression() {
    let content = r#"// @lib: es5
class DataHandler {
    dataArray: Uint8Array;
    loadData(filename) {
        var xmlReq = new XMLHttpRequest();
        xmlReq.open("GET", "/" + filename, true);
        xmlReq.responseType = "arraybuffer";
        xmlReq.onload = function(xmlEvent) {
            /*local*/
            this./*this*/;
        }
    }
}"#;
    let mut s = Session::new_for_test("completionListFunctionExpression", content);
    fourslash::go_to_marker(&mut s, "local");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["xmlEvent"], &[]);
    fourslash::verify_completions_empty_at(&mut s, Some("this"));
}
