use tsox_lsp::fourslash::Session;


#[test]
fn auto_import_file_exclude_patterns4() {
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
export { Event };"#;
    let _s = Session::new_for_test("autoImportFileExcludePatterns4", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
