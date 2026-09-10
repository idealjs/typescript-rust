use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn auto_import_file_exclude_patterns12() {
    let content = r#"// @Filename: /src/vs/test.ts
import { Parts } from './parts';
export class /**/Extended implements Parts {
}
// @Filename: /src/vs/parts.ts
import { Event } from '../thing';
export interface Parts {
	readonly options: Event;
}
// @Filename: /src/event/event.ts
export interface Event {
	(): string;
}
// @Filename: /src/thing.ts
import { Event } from '../event/event';
export { Event };
// @Filename: /src/a.ts
import './thing'
declare module './thing' {
	interface Event {
		c: string;
	}
}"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns12", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
