use tsox_lsp::fourslash::Session;


#[test]
fn completion_import_attributes() {
    let content = r#"
// @target: esnext
// @module: esnext
// @filename: main.ts
import yadda1 from "yadda" with {/*attr*/}
import yadda2 from "yadda" with {attr/*attrEnd1*/: true}
import yadda3 from "yadda" with {attr: /*attrValue*/}

// @filename: yadda
export default {};
"#;
    let _s = Session::new_for_test("completionImportAttributes", content);
    // TODO: f.GoToEachMarker(t, nil, func(marker *fourslash.Marker, index int) {
}
