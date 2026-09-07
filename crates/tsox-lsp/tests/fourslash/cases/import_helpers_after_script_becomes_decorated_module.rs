use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The second diagnostics request forces external helper res"]
#[test]
fn import_helpers_after_script_becomes_decorated_module() {
    let content = r#"// @Filename: /tsconfig.json
{
	"compilerOptions": {
		"target": "es2015",
		"module": "commonjs",
		"experimentalDecorators": true,
		"importHelpers": true
	},
	"files": ["foo.ts"]
}

// @Filename: /foo.ts
declare function dec(value: Function): void;
/*insert*/class C {}

// @Filename: /node_modules/tslib/package.json
{ "name": "tslib", "typings": "tslib.d.ts" }

// @Filename: /node_modules/tslib/tslib.d.ts
export declare function __decorate(...args: any[]): any;

// @Filename: /node_modules/tslib/tslib.js
exports.__decorate = function () {};
"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/foo.ts");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::unsupported("Replace"); // f.Replace(t, f.MarkerByName(t, "insert").Position, 0, `@dec
    // TODO: // The second diagnostics request forces external helper resolution after the edit.
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
}
