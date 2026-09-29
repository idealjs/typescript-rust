use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion11() {
    let content = r#"//@module: commonjs
//@jsx: preserve
//@Filename: exporter.tsx
export class Thing { }
//@Filename: file.tsx
import {Thing} from './exporter';
var x1 = <div></**/"#;
    let mut s = Session::new_for_test("tsxCompletion11", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["Thing"], &[]);
}
