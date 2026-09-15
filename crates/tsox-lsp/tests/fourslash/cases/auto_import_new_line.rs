use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_new_line() {
    let content = r#"// @Filename: /a.ts
export function readFileSync() {}

// @Filename: /b.ts
import {} from "./other1";
import {} from "./other2";


readFileSync/**/"#;
    let _s = Session::new_for_test("autoImportNewLine", content);
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn auto_import_new_line_with_header_comment() {
    let content = r#"// @Filename: /a.ts
export function readFileSync() {}

// @Filename: /b.ts
/* file header comment */
import {} from "./other1";
import {} from "./other2";


readFileSync/**/"#;
    let _s = Session::new_for_test("autoImportNewLineWithHeaderComment", content);
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
