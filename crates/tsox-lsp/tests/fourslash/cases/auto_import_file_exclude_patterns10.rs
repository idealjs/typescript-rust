use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_file_exclude_patterns10() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /src/vs/test.ts
import { Parts } from './parts';
export class /**/Extended implements Parts {
}
// @Filename: /src/vs/parts.ts
import { Event } from '../event/event';

export interface Parts {
	readonly options: Event;
}
// @Filename: /src/event/event.ts
export interface Event {
	(): string;
}
// @Filename: /src/thing.ts
import { Event } from './event/event';
export { Event };
// @Filename: /src/a.ts
import './thing'
declare module './thing' {
	interface Event {
		c: string;
	}
}"#;
    let mut s = Session::new_for_test("autoImportFileExcludePatterns10", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
