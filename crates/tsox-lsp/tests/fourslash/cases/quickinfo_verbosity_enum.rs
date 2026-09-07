use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_enum() {
    let content = r#"// @filename: a.ts
export {};
enum Color/*c*/ {
    Red,
    Green,
    Blue,
}
const x/*x*/: Color = Color.Red;
const enum Direction/*d*/ {
    Up,
    Down,
}
const y/*y*/: Direction = Direction.Up;
enum Flags/*f*/ {
    None = 0,
    IsDirectory = 1 << 0,
    IsFile = 1 << 1,
    IsSymlink = 1 << 2,
}
// @filename: b.ts
export enum Color {
    Red = "red"
}
// @filename: c.ts
import { Color } from "./b";
const c: Color/*a*/ = Color.Red;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"c": {0, 1}, "x": {0, 1}, "d": {0, 1}, "y": {
}
