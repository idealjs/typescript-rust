use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.Insert(t, '}')"]
#[test]
fn space_after_constructor() {
    let content = r#"export class myController {
    private _processId;
    constructor (processId: number) {/*1*/
        this._processId = processId;
    }/*2*/"#;
    let mut s = Session::new_for_test("spaceAfterConstructor", content);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.Insert(t, "}")
}
