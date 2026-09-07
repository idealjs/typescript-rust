use super::*;
use crate::core::tristate::Tristate;

#[test]
fn default_options() {
    let opts = CompilerOptions::default();
    assert_eq!(opts.get_emit_script_target(), ScriptTarget::LATEST_STANDARD);
    assert_eq!(opts.get_emit_module_kind(), ModuleKind::ES2022);
    assert_eq!(
        opts.get_module_resolution_kind(),
        ModuleResolutionKind::Bundler
    );
}

#[test]
fn commonjs_target() {
    let mut opts = CompilerOptions::default();
    opts.target = ScriptTarget::ES5;
    assert_eq!(opts.get_emit_module_kind(), ModuleKind::CommonJS);
}

#[test]
fn node_next_resolution() {
    let mut opts = CompilerOptions::default();
    opts.module = ModuleKind::NodeNext;
    assert_eq!(
        opts.get_module_resolution_kind(),
        ModuleResolutionKind::NodeNext
    );
}

#[test]
fn get_allow_js() {
    let mut opts = CompilerOptions::default();
    assert!(!opts.get_allow_js());
    opts.allow_js = Tristate::True;
    assert!(opts.get_allow_js());
    opts.allow_js = Tristate::Unknown;
    opts.check_js = Tristate::True;
    assert!(opts.get_allow_js());
}

#[test]
fn strict_option_value() {
    let mut opts = CompilerOptions::default();

    assert!(opts.get_strict_option_value(Tristate::Unknown));
    opts.strict = Tristate::True;
    assert!(opts.get_strict_option_value(Tristate::Unknown));
    opts.strict = Tristate::False;
    assert!(!opts.get_strict_option_value(Tristate::Unknown));
    assert!(opts.get_strict_option_value(Tristate::True));
    assert!(!opts.get_strict_option_value(Tristate::False));
}
