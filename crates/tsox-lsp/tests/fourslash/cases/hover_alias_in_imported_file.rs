use tsox_lsp::fourslash::{self, Session};

#[test]
fn hover_alias_in_imported_file() {
    let content = r#"
// @filename: other2.ts
export type SomeAliasType<T> = { value: T };

// @filename: other.ts
import { SomeAliasType } from './other2';

declare function isSomeAliasType(x: any): x is SomeAliasType<any>;

export { isSomeAliasType };

// @filename: main.ts
import { isSomeAliasType } from './other';

export function processValue(value: any) {
  if (/*1*/isSomeAliasType(value)) {
    console.log("ok");
  }
}
"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "(alias) function isSomeAliasType(x: any): x is SomeAliasType<any>",
        "",
    );
}
