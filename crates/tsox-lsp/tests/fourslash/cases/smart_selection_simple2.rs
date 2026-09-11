use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_simple2() {
    let content = r#"export interface IService {
  _serviceBrand: any;

  open(ho/*1*/st: number, data: any): Promise<any>;
  bar(): void/*2*/
}"#;
    let mut s = Session::new_for_test("smartSelection_simple2", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
