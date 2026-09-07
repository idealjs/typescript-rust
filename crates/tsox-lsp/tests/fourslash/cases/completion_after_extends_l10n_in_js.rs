use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GetCompletions"]
#[test]
fn completion_after_extends_l10n_in_js() {
    let content = r#"
// @allowJs: true
// @checkJs: true
// @Filename: /interfaces.d.ts
export interface IL10n {}

// @Filename: /genericl10n.js
/** @typedef {import("./interfaces").IL10n} IL10n */

class L10n {
	constructor(options) {
		this.options = options;
	}
}

/**
 * @implements {IL10n}
 */
class GenericL10n extends L10n/*1*/ {
	constructor(lang) {
		super({ lang });
	}
}

"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("GetCompletions"); // f.GetCompletions(t, nil /*userPreferences*/)
}
