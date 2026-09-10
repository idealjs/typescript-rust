use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_file_exclude_patterns9() {
    // TODO: t.Skip("Known failing fourslash test")
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
// @Filename: /src/vs/test.ts
import { Event } from './event/event';
export { Event };"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns9", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
