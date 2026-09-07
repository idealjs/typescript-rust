use tsox_lsp::fourslash::{self, Session};

#[test]
fn diagnostics_default_import_merged_with_js_doc_type_alias1() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /lib/types.d.ts
export interface RunnerOptions {
  dryRun?: boolean;
}

// @Filename: /lib/runner.js
"use strict";

/**
 * @typedef {import('./types.d.ts').RunnerOptions} RunnerOptions
 */

var EventEmitter = require("node:events").EventEmitter;

class Runner extends EventEmitter {
  constructor() { super(); }
}

module.exports = Runner;

// @Filename: /lib/stats-collector.mjs
/** @typedef {import('./runner.js')} Runner */

import Runner from "./runner.js";

const createStatsCollector = (runner) => runner && Runner;

export { createStatsCollector };
"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/lib/stats-collector.mjs");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 2);
}
