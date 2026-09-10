use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn auto_import_file_exclude_patterns7() {
    let content = r#"// @Filename: /src/vs/workbench/test.ts
import { Parts } from './parts';
export class /**/EditorParts implements Parts { }
// @Filename: /src/vs/event/event.ts
export interface Event {
	(): string;
}
// @Filename: /src/vs/workbench/parts.ts
import { Event } from '../event/event';
export interface Parts {
	readonly options: Event;
}
// @Filename: /src/vs/workbench/workbench.ts
import { Event } from '../event/event';
export { Event };
// @Filename: /src/vs/workbench/workbench2.ts
import { Event } from '../event/event';
export { Event };"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns7", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
