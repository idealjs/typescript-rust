use crate::collections::set::Set;

mod bucket;
mod registry_impl;

pub use bucket::*;
pub use registry_impl::*;

pub fn known_recursive_search_packages() -> Set<String> {
    let mut s = Set::new();
    for pkg in [
        "@material-ui/core",
        "@material-ui/icons",
        "@sap/cds",
        "@testing-library/react-native",
        "ajv",
        "asap",
        "async",
        "aws-sdk",
        "braintree-web",
        "core-js",
        "core-js-pure",
        "crypto-js",
        "cypress-mochawesome-reporter",
        "dd-trace",
        "dumi",
        "dva",
        "egg-mock",
        "electron-log",
        "es-abstract",
        "es6-promise",
        "eslint-config-taro",
        "expo",
        "expo-router",
        "flow-remove-types",
        "gatsby",
        "glamor",
        "gluegun",
        "graphology-indices",
        "graphology-traversal",
        "graphology-utils",
        "jest-expo",
        "lodash",
        "lodash-es",
        "moment",
        "mz",
        "next",
        "pdfjs-dist",
        "protobufjs",
        "react-app-polyfill",
        "react-dev-utils",
        "react-devtools-inline",
        "recast",
        "semver",
        "stylelint-config-html",
        "umi",
        "web3-provider-engine",
        "webpack",
    ] {
        s.add(pkg.to_string());
    }
    s
}
