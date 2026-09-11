use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_multiline_import_brace() {
    let content = r#"import {
	basename,
	extname, joinPath } from '../base/resources.js';
import { URI } from '../base/uri.js';"#;
    let mut s = Session::new_for_test("formatMultilineImportBrace", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"import {
    basename,
    extname, joinPath
} from '../base/resources.js';
import { URI } from '../base/uri.js';"#);
}
