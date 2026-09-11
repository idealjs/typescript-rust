use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_no_crash_multi_source_dts() {
    // TODO: // combined.d.ts has a source map with two sources: a.ts and b.ts.
    // TODO: // The method declaration on line 1 (col 4-end) straddles a source-map boundary:
    // TODO: //   - col 4 ("method") maps to a.ts
    // TODO: //   - col 11 onwards maps to b.ts
    let content = r#"
// @Filename: /a.ts
export {};
// @Filename: /b.ts
export {};
// @Filename: /combined.d.ts
export declare class Bar {
    method(): void;
}
//# sourceMappingURL=combined.d.ts.map
// @Filename: /combined.d.ts.map
{"version":3,"file":"combined.d.ts","sourceRoot":"","sources":["a.ts","b.ts"],"names":[],"mappings":";IAAA,OCAA;AAAA"}
// @Filename: /user.ts
import { Bar } from './combined';
declare const bar: Bar;
bar./*impl*/method();"#;
    let mut s = Session::new_for_test("goToImplementationNoCrashMultiSourceDts", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "impl")
}
