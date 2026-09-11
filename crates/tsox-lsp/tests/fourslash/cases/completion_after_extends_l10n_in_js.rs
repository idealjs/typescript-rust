use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionAfterExtendsL10nInJs", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.GetCompletions(t, nil /*userPreferences*/)
}
