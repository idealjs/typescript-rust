use tsox_lsp::fourslash::{self, Session};


#[test]
fn space_after_constructor() {
    let content = r#"export class myController {
    private _processId;
    constructor (processId: number) {/*1*/
        this._processId = processId;
    }/*2*/"#;
    let mut s = Session::new_for_test("spaceAfterConstructor", content);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, "}");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(processId: number) {"#);
    // TODO: }
}
