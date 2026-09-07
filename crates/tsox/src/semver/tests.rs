use super::*;

#[test]
fn parse_basic_version() {
    let v = try_parse_version("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
    assert!(v.prerelease.is_empty());
    assert!(v.build.is_empty());
}

#[test]
fn parse_prerelease() {
    let v = try_parse_version("1.0.0-alpha.1").unwrap();
    assert_eq!(v.prerelease, vec!["alpha", "1"]);
}

#[test]
fn parse_build() {
    let v = try_parse_version("1.0.0+build.123").unwrap();
    assert_eq!(v.build, vec!["build", "123"]);
}

#[test]
fn parse_partial() {
    let v = try_parse_version("1").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);
}

#[test]
fn parse_invalid() {
    assert!(try_parse_version("1.2.3.4").is_err());
    assert!(try_parse_version("01.2.3").is_err());
    assert!(try_parse_version("abc").is_err());
}

#[test]
fn version_compare() {
    let v1 = try_parse_version("1.0.0").unwrap();
    let v2 = try_parse_version("2.0.0").unwrap();
    let v3 = try_parse_version("1.0.0").unwrap();
    assert_eq!(v1.compare(&v2), Ordering::Less);
    assert_eq!(v2.compare(&v1), Ordering::Greater);
    assert_eq!(v1.compare(&v3), Ordering::Equal);
}

#[test]
fn prerelease_compare() {
    let v1 = try_parse_version("1.0.0-alpha").unwrap();
    let v2 = try_parse_version("1.0.0").unwrap();
    assert_eq!(v1.compare(&v2), Ordering::Less);
}

#[test]
fn version_display() {
    let v = try_parse_version("1.2.3-alpha.1+build.42").unwrap();
    assert_eq!(v.to_string(), "1.2.3-alpha.1+build.42");
}

#[test]
fn range_exact() {
    let range = try_parse_version_range("1.2.3").unwrap();
    assert!(range.test(&try_parse_version("1.2.3").unwrap()));
    assert!(!range.test(&try_parse_version("1.2.4").unwrap()));
}

#[test]
fn range_caret() {
    let range = try_parse_version_range("^1.2.3").unwrap();
    assert!(range.test(&try_parse_version("1.2.5").unwrap()));
    assert!(range.test(&try_parse_version("1.9.0").unwrap()));
    assert!(!range.test(&try_parse_version("2.0.0").unwrap()));
    assert!(!range.test(&try_parse_version("1.2.2").unwrap()));
}

#[test]
fn range_tilde() {
    let range = try_parse_version_range("~1.2.3").unwrap();
    assert!(range.test(&try_parse_version("1.2.5").unwrap()));
    assert!(!range.test(&try_parse_version("1.3.0").unwrap()));
}

#[test]
fn range_or() {
    let range = try_parse_version_range("1.0.0 || 2.0.0").unwrap();
    assert!(range.test(&try_parse_version("1.0.0").unwrap()));
    assert!(range.test(&try_parse_version("2.0.0").unwrap()));
    assert!(!range.test(&try_parse_version("3.0.0").unwrap()));
}

#[test]
fn range_wildcard() {
    let range = try_parse_version_range("1.x").unwrap();
    assert!(range.test(&try_parse_version("1.0.0").unwrap()));
    assert!(range.test(&try_parse_version("1.5.3").unwrap()));
    assert!(!range.test(&try_parse_version("2.0.0").unwrap()));
}

use std::cmp::Ordering;

const LT: Ordering = Ordering::Less;
const EQ: Ordering = Ordering::Equal;
const GT: Ordering = Ordering::Greater;

#[test]
fn test_try_parse_semver() {
    let tests: &[(&str, Version)] = &[
        (
            "1.2.3-pre.4+build.5",
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec!["pre".into(), "4".into()],
                build: vec!["build".into(), "5".into()],
            },
        ),
        (
            "1.2.3-pre.4",
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec!["pre".into(), "4".into()],
                build: vec![],
            },
        ),
        (
            "1.2.3+build.4",
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec![],
                build: vec!["build".into(), "4".into()],
            },
        ),
        (
            "1.2.3",
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec![],
                build: vec![],
            },
        ),
    ];
    for (input, expected) in tests {
        let v = try_parse_version(input)
            .unwrap_or_else(|e| panic!("TryParseVersion({:?}): {}", input, e));
        assert_eq!(v.major, expected.major, "{}: major", input);
        assert_eq!(v.minor, expected.minor, "{}: minor", input);
        assert_eq!(v.patch, expected.patch, "{}: patch", input);
        assert_eq!(v.prerelease, expected.prerelease, "{}: prerelease", input);
        assert_eq!(v.build, expected.build, "{}: build", input);
    }
}

#[test]
fn test_version_string() {
    let tests: &[(Version, &str)] = &[
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec!["pre".into(), "4".into()],
                build: vec!["build".into(), "5".into()],
            },
            "1.2.3-pre.4+build.5",
        ),
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec!["pre".into(), "4".into()],
                build: vec!["build".into()],
            },
            "1.2.3-pre.4+build",
        ),
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec![],
                build: vec!["build".into()],
            },
            "1.2.3+build",
        ),
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec!["pre".into(), "4".into()],
                build: vec![],
            },
            "1.2.3-pre.4",
        ),
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec![],
                build: vec!["build".into(), "4".into()],
            },
            "1.2.3+build.4",
        ),
        (
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: vec![],
                build: vec![],
            },
            "1.2.3",
        ),
    ];
    for (version, expected) in tests {
        assert_eq!(version.to_string(), *expected);
    }
}

#[test]
fn test_version_compare() {
    let tests: &[(&str, &str, Ordering)] = &[
        ("1.0.0", "2.0.0", LT),
        ("1.0.0", "1.1.0", LT),
        ("1.0.0", "1.0.1", LT),
        ("2.0.0", "1.0.0", GT),
        ("1.1.0", "1.0.0", GT),
        ("1.0.1", "1.0.0", GT),
        ("1.0.0", "1.0.0", EQ),
        ("1.0.0", "1.0.0-pre", GT),
        ("1.0.1-pre", "1.0.0", GT),
        ("1.0.0-pre", "1.0.0", LT),
        ("1.0.0-0", "1.0.0-1", LT),
        ("1.0.0-1", "1.0.0-0", GT),
        ("1.0.0-2", "1.0.0-10", LT),
        ("1.0.0-10", "1.0.0-2", GT),
        ("1.0.0-0", "1.0.0-0", EQ),
        ("1.0.0-a", "1.0.0-b", LT),
        ("1.0.0-a-2", "1.0.0-a-10", GT),
        ("1.0.0-b", "1.0.0-a", GT),
        ("1.0.0-a", "1.0.0-a", EQ),
        ("1.0.0-A", "1.0.0-a", LT),
        ("1.0.0-0", "1.0.0-alpha", LT),
        ("1.0.0-alpha", "1.0.0-0", GT),
        ("1.0.0-0", "1.0.0-0", EQ),
        ("1.0.0-alpha", "1.0.0-alpha", EQ),
        ("1.0.0-alpha", "1.0.0-alpha.0", LT),
        ("1.0.0-alpha.0", "1.0.0-alpha", GT),
        ("1.0.0-a.0.b.1", "1.0.0-a.0.b.2", LT),
        ("1.0.0-a.0.b.1", "1.0.0-b.0.a.1", LT),
        ("1.0.0-a.0.b.2", "1.0.0-a.0.b.1", GT),
        ("1.0.0-b.0.a.1", "1.0.0-a.0.b.1", GT),
        ("1.0.0+build", "1.0.0", EQ),
        ("1.0.0+build.stuff", "1.0.0", EQ),
        ("1.0.0", "1.0.0+build", EQ),
        ("1.0.0+build", "1.0.0+stuff", EQ),
        ("1.0.0-alpha.99999", "1.0.0-alpha.100000", LT),
        ("1.0.0-alpha.beta", "1.0.0-alpha.alpha", GT),
    ];
    for (v1s, v2s, want) in tests {
        let v1 = try_parse_version(v1s).unwrap();
        let v2 = try_parse_version(v2s).unwrap();
        assert_eq!(v1.compare(&v2), *want, "{} <=> {}", v1s, v2s);
    }
}

fn assert_ranges_good_bad(version_range_string: &str, good: &[&str], bad: &[&str]) {
    let version_range = try_parse_version_range(version_range_string)
        .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", version_range_string));
    for g in good {
        let v = try_parse_version(g).unwrap();
        assert!(
            version_range.test(&v),
            "{} should be matched by range {}",
            g,
            version_range_string
        );
    }
    for b in bad {
        let v = try_parse_version(b).unwrap();
        assert!(
            !version_range.test(&v),
            "{} should not be matched by range {}",
            b,
            version_range_string
        );
    }
}

fn assert_range_test(name: &str, range_text: &str, version_text: &str, in_range: bool) {
    let version_range = try_parse_version_range(range_text)
        .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", range_text));
    let version = try_parse_version(version_text)
        .unwrap_or_else(|e| panic!("TryParseVersion({:?}): {}", version_text, e));
    assert_eq!(
        version_range.test(&version),
        in_range,
        "{}: version {} in range {} == {}",
        name,
        version_text,
        range_text,
        in_range
    );
}

#[test]
fn test_wildcards_have_same_string() {
    fn assert_all_identical(name: &str, strs: &[&str]) {
        for &s1 in strs {
            for &s2 in strs {
                let v1 = try_parse_version_range(s1)
                    .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", s1));
                let v2 = try_parse_version_range(s2)
                    .unwrap_or_else(|| panic!("TryParseVersionRange({:?}) failed", s2));
                assert_eq!(v1.to_string(), v2.to_string(), "{}: {} == {}", name, s1, s2);
            }
        }
    }
    let major = [
        "", "*", "*.*", "*.*.*", "x", "x.x", "x.x.x", "X", "X.X", "X.X.X",
    ];
    let minor = ["1", "1.*", "1.*.*", "1.x", "1.x.x", "1.X", "1.X.X"];
    let patch = ["1.2", "1.2.*", "1.2.x", "1.2.X"];
    let mixed = ["x", "X", "*", "x.X.x", "X.x.*"];
    assert_all_identical("majorWildcardStrings", &major);
    assert_all_identical("minorWildcardStrings", &minor);
    assert_all_identical("patchWildcardStrings", &patch);
    assert_all_identical("mixedCaseWildcardStrings", &mixed);
}

#[test]
fn test_version_ranges() {
    assert_ranges_good_bad(
        "1",
        &["1.0.0", "1.9.9", "1.0.0-pre", "1.0.0+build"],
        &["0.0.0", "2.0.0", "0.0.0-pre", "0.0.0+build"],
    );
    assert_ranges_good_bad(
        "1.2",
        &["1.2.0", "1.2.9", "1.2.0-pre", "1.2.0+build"],
        &["1.1.0", "1.3.0", "1.1.0-pre", "1.1.0+build"],
    );
    assert_ranges_good_bad(
        "1.2.3",
        &["1.2.3", "1.2.3+build"],
        &["1.2.2", "1.2.4", "1.2.2-pre", "1.2.2+build", "1.2.3-pre"],
    );
    assert_ranges_good_bad(
        "1.2.3-pre",
        &["1.2.3-pre", "1.2.3-pre+build.stuff"],
        &[
            "1.2.3",
            "1.2.3-pre.0",
            "1.2.3-pre.9",
            "1.2.3-pre.0+build",
            "1.2.3-pre.9+build",
            "1.2.3+build",
            "1.2.4",
        ],
    );
    assert_ranges_good_bad("<3.8.0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);
    assert_ranges_good_bad("<=3.8.0", &["3.6", "3.7", "3.8"], &["3.9", "4.0"]);
    assert_ranges_good_bad(">3.8.0", &["3.9", "4.0"], &["3.6", "3.7", "3.8"]);
    assert_ranges_good_bad(">=3.8.0", &["3.8", "3.9", "4.0"], &["3.6", "3.7"]);
    assert_ranges_good_bad("<3.8.0-0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);
    assert_ranges_good_bad("<=3.8.0-0", &["3.6", "3.7"], &["3.8", "3.9", "4.0"]);

    let lotsa_ones = "1".repeat(320);
    let range_str = format!(">=1.2.3-1{}", lotsa_ones);
    let g0 = format!("1.2.3-1{}", lotsa_ones);
    let g1 = format!("1.2.3-11{}{}.1", lotsa_ones, "");
    let g2 = format!("1.2.3-1{}.1+build", lotsa_ones);
    let b0 = format!("1.2.3-{}.1+build", lotsa_ones);
    assert_ranges_good_bad(&range_str, &[&g0, &g1, &g2], &[&b0]);
}

#[test]
fn test_comparators_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        ("", "2.0.0", true),
        ("", "2.0.0-0", true),
        ("", "1.1.0", true),
        ("", "1.1.0-0", true),
        ("", "1.0.1", true),
        ("", "1.0.1-0", true),
        ("", "1.0.0", true),
        ("", "1.0.0-0", true),
        ("", "0.0.0", true),
        ("", "0.0.0-0", true),
        ("*", "2.0.0", true),
        ("*", "2.0.0-0", true),
        ("*", "1.1.0", true),
        ("*", "1.1.0-0", true),
        ("*", "1.0.1", true),
        ("*", "1.0.1-0", true),
        ("*", "1.0.0", true),
        ("*", "1.0.0-0", true),
        ("*", "0.0.0", true),
        ("*", "0.0.0-0", true),
        ("1", "2.0.0", false),
        ("1", "2.0.0-0", false),
        ("1", "1.1.0", true),
        ("1", "1.1.0-0", true),
        ("1", "1.0.1", true),
        ("1", "1.0.1-0", true),
        ("1", "1.0.0", true),
        ("1", "1.0.0-0", true),
        ("1", "0.0.0", false),
        ("1", "0.0.0-0", false),
        ("1.1", "2.0.0", false),
        ("1.1", "2.0.0-0", false),
        ("1.1", "1.1.0", true),
        ("1.1", "1.1.0-0", true),
        ("1.1", "1.0.1", false),
        ("1.1", "1.0.1-0", false),
        ("1.1", "1.0.0", false),
        ("1.1", "1.0.0-0", false),
        ("1.1", "0.0.0", false),
        ("1.1", "0.0.0-0", false),
        ("1.0", "2.0.0", false),
        ("1.0", "2.0.0-0", false),
        ("1.0", "1.1.0", false),
        ("1.0", "1.1.0-0", false),
        ("1.0", "1.0.1", true),
        ("1.0", "1.0.1-0", true),
        ("1.0", "1.0.0", true),
        ("1.0", "1.0.0-0", true),
        ("1.0", "0.0.0", false),
        ("1.0", "0.0.0-0", false),
        ("1.1.0", "2.0.0", false),
        ("1.1.0", "2.0.0-0", false),
        ("1.1.0", "1.1.0", true),
        ("1.1.0", "1.1.0-0", false),
        ("1.1.0", "1.0.1", false),
        ("1.1.0", "1.0.1-0", false),
        ("1.1.0", "1.0.0-0", false),
        ("1.1.0", "1.0.0", false),
        ("1.1.0", "0.0.0", false),
        ("1.1.0", "0.0.0-0", false),
        ("1.1.0-0", "2.0.0", false),
        ("1.1.0-0", "2.0.0-0", false),
        ("1.1.0-0", "1.1.0", false),
        ("1.1.0-0", "1.1.0-0", true),
        ("1.1.0-0", "1.0.1", false),
        ("1.1.0-0", "1.0.1-0", false),
        ("1.1.0-0", "1.0.0-0", false),
        ("1.1.0-0", "1.0.0", false),
        ("1.1.0-0", "0.0.0", false),
        ("1.1.0-0", "0.0.0-0", false),
        ("1.0.1", "2.0.0", false),
        ("1.0.1", "2.0.0-0", false),
        ("1.0.1", "1.1.0", false),
        ("1.0.1", "1.1.0-0", false),
        ("1.0.1", "1.0.1", true),
        ("1.0.1", "1.0.1-0", false),
        ("1.0.1", "1.0.0-0", false),
        ("1.0.1", "1.0.0", false),
        ("1.0.1", "0.0.0", false),
        ("1.0.1", "0.0.0-0", false),
        ("1.0.1-0", "2.0.0", false),
        ("1.0.1-0", "2.0.0-0", false),
        ("1.0.1-0", "1.1.0", false),
        ("1.0.1-0", "1.1.0-0", false),
        ("1.0.1-0", "1.0.1", false),
        ("1.0.1-0", "1.0.1-0", true),
        ("1.0.1-0", "1.0.0-0", false),
        ("1.0.1-0", "1.0.0", false),
        ("1.0.1-0", "0.0.0", false),
        ("1.0.1-0", "0.0.0-0", false),
        ("1.0.0", "2.0.0", false),
        ("1.0.0", "2.0.0-0", false),
        ("1.0.0", "1.1.0", false),
        ("1.0.0", "1.1.0-0", false),
        ("1.0.0", "1.0.1", false),
        ("1.0.0", "1.0.1-0", false),
        ("1.0.0", "1.0.0-0", false),
        ("1.0.0", "1.0.0", true),
        ("1.0.0", "0.0.0", false),
        ("1.0.0", "0.0.0-0", false),
        ("1.0.0-0", "2.0.0", false),
        ("1.0.0-0", "2.0.0-0", false),
        ("1.0.0-0", "1.1.0", false),
        ("1.0.0-0", "1.1.0-0", false),
        ("1.0.0-0", "1.0.1", false),
        ("1.0.0-0", "1.0.1-0", false),
        ("1.0.0-0", "1.0.0", false),
        ("1.0.0-0", "1.0.0-0", true),
        ("=*", "2.0.0", true),
        ("=*", "2.0.0-0", true),
        ("=*", "1.1.0", true),
        ("=*", "1.1.0-0", true),
        ("=*", "1.0.1", true),
        ("=*", "1.0.1-0", true),
        ("=*", "1.0.0", true),
        ("=*", "1.0.0-0", true),
        ("=*", "0.0.0", true),
        ("=*", "0.0.0-0", true),
        ("=1", "2.0.0", false),
        ("=1", "2.0.0-0", false),
        ("=1", "1.1.0", true),
        ("=1", "1.1.0-0", true),
        ("=1", "1.0.1", true),
        ("=1", "1.0.1-0", true),
        ("=1", "1.0.0", true),
        ("=1", "1.0.0-0", true),
        ("=1", "0.0.0", false),
        ("=1", "0.0.0-0", false),
        ("=1.1", "2.0.0", false),
        ("=1.1", "2.0.0-0", false),
        ("=1.1", "1.1.0", true),
        ("=1.1", "1.1.0-0", true),
        ("=1.1", "1.0.1", false),
        ("=1.1", "1.0.1-0", false),
        ("=1.1", "1.0.0", false),
        ("=1.1", "1.0.0-0", false),
        ("=1.1", "0.0.0", false),
        ("=1.1", "0.0.0-0", false),
        ("=1.0", "2.0.0", false),
        ("=1.0", "2.0.0-0", false),
        ("=1.0", "1.1.0", false),
        ("=1.0", "1.1.0-0", false),
        ("=1.0", "1.0.1", true),
        ("=1.0", "1.0.1-0", true),
        ("=1.0", "1.0.0", true),
        ("=1.0", "1.0.0-0", true),
        ("=1.0", "0.0.0", false),
        ("=1.0", "0.0.0-0", false),
        ("=1.1.0", "2.0.0", false),
        ("=1.1.0", "2.0.0-0", false),
        ("=1.1.0", "1.1.0", true),
        ("=1.1.0", "1.1.0-0", false),
        ("=1.1.0", "1.0.1", false),
        ("=1.1.0", "1.0.1-0", false),
        ("=1.1.0", "1.0.0-0", false),
        ("=1.1.0", "1.0.0", false),
        ("=1.1.0", "0.0.0", false),
        ("=1.1.0", "0.0.0-0", false),
        ("=1.1.0-0", "2.0.0", false),
        ("=1.1.0-0", "2.0.0-0", false),
        ("=1.1.0-0", "1.1.0", false),
        ("=1.1.0-0", "1.1.0-0", true),
        ("=1.1.0-0", "1.0.1", false),
        ("=1.1.0-0", "1.0.1-0", false),
        ("=1.1.0-0", "1.0.0-0", false),
        ("=1.1.0-0", "1.0.0", false),
        ("=1.1.0-0", "0.0.0", false),
        ("=1.1.0-0", "0.0.0-0", false),
        ("=1.0.1", "2.0.0", false),
        ("=1.0.1", "2.0.0-0", false),
        ("=1.0.1", "1.1.0", false),
        ("=1.0.1", "1.1.0-0", false),
        ("=1.0.1", "1.0.1", true),
        ("=1.0.1", "1.0.1-0", false),
        ("=1.0.1", "1.0.0-0", false),
        ("=1.0.1", "1.0.0", false),
        ("=1.0.1", "0.0.0", false),
        ("=1.0.1", "0.0.0-0", false),
        ("=1.0.1-0", "2.0.0", false),
        ("=1.0.1-0", "2.0.0-0", false),
        ("=1.0.1-0", "1.1.0", false),
        ("=1.0.1-0", "1.1.0-0", false),
        ("=1.0.1-0", "1.0.1", false),
        ("=1.0.1-0", "1.0.1-0", true),
        ("=1.0.1-0", "1.0.0-0", false),
        ("=1.0.1-0", "1.0.0", false),
        ("=1.0.1-0", "0.0.0", false),
        ("=1.0.1-0", "0.0.0-0", false),
        ("=1.0.0", "2.0.0", false),
        ("=1.0.0", "2.0.0-0", false),
        ("=1.0.0", "1.1.0", false),
        ("=1.0.0", "1.1.0-0", false),
        ("=1.0.0", "1.0.1", false),
        ("=1.0.0", "1.0.1-0", false),
        ("=1.0.0", "1.0.0-0", false),
        ("=1.0.0", "1.0.0", true),
        ("=1.0.0", "0.0.0", false),
        ("=1.0.0", "0.0.0-0", false),
        ("=1.0.0-0", "2.0.0", false),
        ("=1.0.0-0", "2.0.0-0", false),
        ("=1.0.0-0", "1.1.0", false),
        ("=1.0.0-0", "1.1.0-0", false),
        ("=1.0.0-0", "1.0.1", false),
        ("=1.0.0-0", "1.0.1-0", false),
        ("=1.0.0-0", "1.0.0", false),
        ("=1.0.0-0", "1.0.0-0", true),
        (">*", "2.0.0", false),
        (">*", "2.0.0-0", false),
        (">*", "1.1.0", false),
        (">*", "1.1.0-0", false),
        (">*", "1.0.1", false),
        (">*", "1.0.1-0", false),
        (">*", "1.0.0", false),
        (">*", "1.0.0-0", false),
        (">*", "0.0.0", false),
        (">*", "0.0.0-0", false),
        (">1", "2.0.0", true),
        (">1", "2.0.0-0", true),
        (">1", "1.1.0", false),
        (">1", "1.1.0-0", false),
        (">1", "1.0.1", false),
        (">1", "1.0.1-0", false),
        (">1", "1.0.0", false),
        (">1", "1.0.0-0", false),
        (">1", "0.0.0", false),
        (">1", "0.0.0-0", false),
        (">1.1", "2.0.0", true),
        (">1.1", "2.0.0-0", true),
        (">1.1", "1.1.0", false),
        (">1.1", "1.1.0-0", false),
        (">1.1", "1.0.1", false),
        (">1.1", "1.0.1-0", false),
        (">1.1", "1.0.0", false),
        (">1.1", "1.0.0-0", false),
        (">1.1", "0.0.0", false),
        (">1.1", "0.0.0-0", false),
        (">1.0", "2.0.0", true),
        (">1.0", "2.0.0-0", true),
        (">1.0", "1.1.0", true),
        (">1.0", "1.1.0-0", true),
        (">1.0", "1.0.1", false),
        (">1.0", "1.0.1-0", false),
        (">1.0", "1.0.0", false),
        (">1.0", "1.0.0-0", false),
        (">1.0", "0.0.0", false),
        (">1.0", "0.0.0-0", false),
        (">1.1.0", "2.0.0", true),
        (">1.1.0", "2.0.0-0", true),
        (">1.1.0", "1.1.0", false),
        (">1.1.0", "1.1.0-0", false),
        (">1.1.0", "1.0.1", false),
        (">1.1.0", "1.0.1-0", false),
        (">1.1.0", "1.0.0", false),
        (">1.1.0", "1.0.0-0", false),
        (">1.1.0", "0.0.0", false),
        (">1.1.0", "0.0.0-0", false),
        (">1.1.0-0", "2.0.0", true),
        (">1.1.0-0", "2.0.0-0", true),
        (">1.1.0-0", "1.1.0", true),
        (">1.1.0-0", "1.1.0-0", false),
        (">1.1.0-0", "1.0.1", false),
        (">1.1.0-0", "1.0.1-0", false),
        (">1.1.0-0", "1.0.0", false),
        (">1.1.0-0", "1.0.0-0", false),
        (">1.1.0-0", "0.0.0", false),
        (">1.1.0-0", "0.0.0-0", false),
        (">1.0.1", "2.0.0", true),
        (">1.0.1", "2.0.0-0", true),
        (">1.0.1", "1.1.0", true),
        (">1.0.1", "1.1.0-0", true),
        (">1.0.1", "1.0.1", false),
        (">1.0.1", "1.0.1-0", false),
        (">1.0.1", "1.0.0", false),
        (">1.0.1", "1.0.0-0", false),
        (">1.0.1", "0.0.0", false),
        (">1.0.1", "0.0.0-0", false),
        (">1.0.1-0", "2.0.0", true),
        (">1.0.1-0", "2.0.0-0", true),
        (">1.0.1-0", "1.1.0", true),
        (">1.0.1-0", "1.1.0-0", true),
        (">1.0.1-0", "1.0.1", true),
        (">1.0.1-0", "1.0.1-0", false),
        (">1.0.1-0", "1.0.0", false),
        (">1.0.1-0", "1.0.0-0", false),
        (">1.0.1-0", "0.0.0", false),
        (">1.0.1-0", "0.0.0-0", false),
        (">1.0.0", "2.0.0", true),
        (">1.0.0", "2.0.0-0", true),
        (">1.0.0", "1.1.0", true),
        (">1.0.0", "1.1.0-0", true),
        (">1.0.0", "1.0.1", true),
        (">1.0.0", "1.0.1-0", true),
        (">1.0.0", "1.0.0", false),
        (">1.0.0", "1.0.0-0", false),
        (">1.0.0", "0.0.0", false),
        (">1.0.0", "0.0.0-0", false),
        (">1.0.0-0", "2.0.0", true),
        (">1.0.0-0", "2.0.0-0", true),
        (">1.0.0-0", "1.1.0", true),
        (">1.0.0-0", "1.1.0-0", true),
        (">1.0.0-0", "1.0.1", true),
        (">1.0.0-0", "1.0.1-0", true),
        (">1.0.0-0", "1.0.0", true),
        (">1.0.0-0", "1.0.0-0", false),
        (">1.0.0-0", "0.0.0", false),
        (">1.0.0-0", "0.0.0-0", false),
        (">=*", "2.0.0", true),
        (">=*", "2.0.0-0", true),
        (">=*", "1.1.0", true),
        (">=*", "1.1.0-0", true),
        (">=*", "1.0.1", true),
        (">=*", "1.0.1-0", true),
        (">=*", "1.0.0", true),
        (">=*", "1.0.0-0", true),
        (">=*", "0.0.0", true),
        (">=*", "0.0.0-0", true),
        (">=1", "2.0.0", true),
        (">=1", "2.0.0-0", true),
        (">=1", "1.1.0", true),
        (">=1", "1.1.0-0", true),
        (">=1", "1.0.1", true),
        (">=1", "1.0.1-0", true),
        (">=1", "1.0.0", true),
        (">=1", "1.0.0-0", true),
        (">=1", "0.0.0", false),
        (">=1", "0.0.0-0", false),
        (">=1.1", "2.0.0", true),
        (">=1.1", "2.0.0-0", true),
        (">=1.1", "1.1.0", true),
        (">=1.1", "1.1.0-0", true),
        (">=1.1", "1.0.1", false),
        (">=1.1", "1.0.1-0", false),
        (">=1.1", "1.0.0", false),
        (">=1.1", "1.0.0-0", false),
        (">=1.1", "0.0.0", false),
        (">=1.1", "0.0.0-0", false),
        (">=1.0", "2.0.0", true),
        (">=1.0", "2.0.0-0", true),
        (">=1.0", "1.1.0", true),
        (">=1.0", "1.1.0-0", true),
        (">=1.0", "1.0.1", true),
        (">=1.0", "1.0.1-0", true),
        (">=1.0", "1.0.0", true),
        (">=1.0", "1.0.0-0", true),
        (">=1.0", "0.0.0", false),
        (">=1.0", "0.0.0-0", false),
        (">=1.1.0", "2.0.0", true),
        (">=1.1.0", "2.0.0-0", true),
        (">=1.1.0", "1.1.0", true),
        (">=1.1.0", "1.1.0-0", false),
        (">=1.1.0", "1.0.1", false),
        (">=1.1.0", "1.0.1-0", false),
        (">=1.1.0", "1.0.0", false),
        (">=1.1.0", "1.0.0-0", false),
        (">=1.1.0", "0.0.0", false),
        (">=1.1.0", "0.0.0-0", false),
        (">=1.1.0-0", "2.0.0", true),
        (">=1.1.0-0", "2.0.0-0", true),
        (">=1.1.0-0", "1.1.0", true),
        (">=1.1.0-0", "1.1.0-0", true),
        (">=1.1.0-0", "1.0.1", false),
        (">=1.1.0-0", "1.0.1-0", false),
        (">=1.1.0-0", "1.0.0", false),
        (">=1.1.0-0", "1.0.0-0", false),
        (">=1.1.0-0", "0.0.0", false),
        (">=1.1.0-0", "0.0.0-0", false),
        (">=1.0.1", "2.0.0", true),
        (">=1.0.1", "2.0.0-0", true),
        (">=1.0.1", "1.1.0", true),
        (">=1.0.1", "1.1.0-0", true),
        (">=1.0.1", "1.0.1", true),
        (">=1.0.1", "1.0.1-0", false),
        (">=1.0.1", "1.0.0", false),
        (">=1.0.1", "1.0.0-0", false),
        (">=1.0.1", "0.0.0", false),
        (">=1.0.1", "0.0.0-0", false),
        (">=1.0.1-0", "2.0.0", true),
        (">=1.0.1-0", "2.0.0-0", true),
        (">=1.0.1-0", "1.1.0", true),
        (">=1.0.1-0", "1.1.0-0", true),
        (">=1.0.1-0", "1.0.1", true),
        (">=1.0.1-0", "1.0.1-0", true),
        (">=1.0.1-0", "1.0.0", false),
        (">=1.0.1-0", "1.0.0-0", false),
        (">=1.0.1-0", "0.0.0", false),
        (">=1.0.1-0", "0.0.0-0", false),
        (">=1.0.0", "2.0.0", true),
        (">=1.0.0", "2.0.0-0", true),
        (">=1.0.0", "1.1.0", true),
        (">=1.0.0", "1.1.0-0", true),
        (">=1.0.0", "1.0.1", true),
        (">=1.0.0", "1.0.1-0", true),
        (">=1.0.0", "1.0.0", true),
        (">=1.0.0", "1.0.0-0", false),
        (">=1.0.0", "0.0.0", false),
        (">=1.0.0", "0.0.0-0", false),
        (">=1.0.0-0", "2.0.0", true),
        (">=1.0.0-0", "2.0.0-0", true),
        (">=1.0.0-0", "1.1.0", true),
        (">=1.0.0-0", "1.1.0-0", true),
        (">=1.0.0-0", "1.0.1", true),
        (">=1.0.0-0", "1.0.1-0", true),
        (">=1.0.0-0", "1.0.0", true),
        (">=1.0.0-0", "1.0.0-0", true),
        (">=1.0.0-0", "0.0.0", false),
        (">=1.0.0-0", "0.0.0-0", false),
        ("<*", "2.0.0", false),
        ("<*", "2.0.0-0", false),
        ("<*", "1.1.0", false),
        ("<*", "1.1.0-0", false),
        ("<*", "1.0.1", false),
        ("<*", "1.0.1-0", false),
        ("<*", "1.0.0", false),
        ("<*", "1.0.0-0", false),
        ("<*", "0.0.0", false),
        ("<*", "0.0.0-0", false),
        ("<1", "2.0.0", false),
        ("<1", "2.0.0-0", false),
        ("<1", "1.1.0", false),
        ("<1", "1.1.0-0", false),
        ("<1", "1.0.1", false),
        ("<1", "1.0.1-0", false),
        ("<1", "1.0.0", false),
        ("<1", "1.0.0-0", false),
        ("<1", "0.0.0", true),
        ("<1", "0.0.0-0", true),
        ("<1.1", "2.0.0", false),
        ("<1.1", "2.0.0-0", false),
        ("<1.1", "1.1.0", false),
        ("<1.1", "1.1.0-0", false),
        ("<1.1", "1.0.1", true),
        ("<1.1", "1.0.1-0", true),
        ("<1.1", "1.0.0", true),
        ("<1.1", "1.0.0-0", true),
        ("<1.1", "0.0.0", true),
        ("<1.1", "0.0.0-0", true),
        ("<1.0", "2.0.0", false),
        ("<1.0", "2.0.0-0", false),
        ("<1.0", "1.1.0", false),
        ("<1.0", "1.1.0-0", false),
        ("<1.0", "1.0.1", false),
        ("<1.0", "1.0.1-0", false),
        ("<1.0", "1.0.0", false),
        ("<1.0", "1.0.0-0", false),
        ("<1.0", "0.0.0", true),
        ("<1.0", "0.0.0-0", true),
        ("<1.1.0", "2.0.0", false),
        ("<1.1.0", "2.0.0-0", false),
        ("<1.1.0", "1.1.0", false),
        ("<1.1.0", "1.1.0-0", true),
        ("<1.1.0", "1.0.1", true),
        ("<1.1.0", "1.0.1-0", true),
        ("<1.1.0", "1.0.0", true),
        ("<1.1.0", "1.0.0-0", true),
        ("<1.1.0", "0.0.0", true),
        ("<1.1.0", "0.0.0-0", true),
        ("<1.1.0-0", "2.0.0", false),
        ("<1.1.0-0", "2.0.0-0", false),
        ("<1.1.0-0", "1.1.0", false),
        ("<1.1.0-0", "1.1.0-0", false),
        ("<1.1.0-0", "1.0.1", true),
        ("<1.1.0-0", "1.0.1-0", true),
        ("<1.1.0-0", "1.0.0", true),
        ("<1.1.0-0", "1.0.0-0", true),
        ("<1.1.0-0", "0.0.0", true),
        ("<1.1.0-0", "0.0.0-0", true),
        ("<1.0.1", "2.0.0", false),
        ("<1.0.1", "2.0.0-0", false),
        ("<1.0.1", "1.1.0", false),
        ("<1.0.1", "1.1.0-0", false),
        ("<1.0.1", "1.0.1", false),
        ("<1.0.1", "1.0.1-0", true),
        ("<1.0.1", "1.0.0", true),
        ("<1.0.1", "1.0.0-0", true),
        ("<1.0.1", "0.0.0", true),
        ("<1.0.1", "0.0.0-0", true),
        ("<1.0.1-0", "2.0.0", false),
        ("<1.0.1-0", "2.0.0-0", false),
        ("<1.0.1-0", "1.1.0", false),
        ("<1.0.1-0", "1.1.0-0", false),
        ("<1.0.1-0", "1.0.1", false),
        ("<1.0.1-0", "1.0.1-0", false),
        ("<1.0.1-0", "1.0.0", true),
        ("<1.0.1-0", "1.0.0-0", true),
        ("<1.0.1-0", "0.0.0", true),
        ("<1.0.1-0", "0.0.0-0", true),
        ("<1.0.0", "2.0.0", false),
        ("<1.0.0", "2.0.0-0", false),
        ("<1.0.0", "1.1.0", false),
        ("<1.0.0", "1.1.0-0", false),
        ("<1.0.0", "1.0.1", false),
        ("<1.0.0", "1.0.1-0", false),
        ("<1.0.0", "1.0.0", false),
        ("<1.0.0", "1.0.0-0", true),
        ("<1.0.0", "0.0.0", true),
        ("<1.0.0", "0.0.0-0", true),
        ("<1.0.0-0", "2.0.0", false),
        ("<1.0.0-0", "2.0.0-0", false),
        ("<1.0.0-0", "1.1.0", false),
        ("<1.0.0-0", "1.1.0-0", false),
        ("<1.0.0-0", "1.0.1", false),
        ("<1.0.0-0", "1.0.1-0", false),
        ("<1.0.0-0", "1.0.0", false),
        ("<1.0.0-0", "1.0.0-0", false),
        ("<1.0.0-0", "0.0.0", true),
        ("<1.0.0-0", "0.0.0-0", true),
        ("<=*", "2.0.0", true),
        ("<=*", "2.0.0-0", true),
        ("<=*", "1.1.0", true),
        ("<=*", "1.1.0-0", true),
        ("<=*", "1.0.1", true),
        ("<=*", "1.0.1-0", true),
        ("<=*", "1.0.0", true),
        ("<=*", "1.0.0-0", true),
        ("<=*", "0.0.0", true),
        ("<=*", "0.0.0-0", true),
        ("<=1", "2.0.0", false),
        ("<=1", "2.0.0-0", false),
        ("<=1", "1.1.0", true),
        ("<=1", "1.1.0-0", true),
        ("<=1", "1.0.1", true),
        ("<=1", "1.0.1-0", true),
        ("<=1", "1.0.0", true),
        ("<=1", "1.0.0-0", true),
        ("<=1", "0.0.0", true),
        ("<=1", "0.0.0-0", true),
        ("<=1.1", "2.0.0", false),
        ("<=1.1", "2.0.0-0", false),
        ("<=1.1", "1.1.0", true),
        ("<=1.1", "1.1.0-0", true),
        ("<=1.1", "1.0.1", true),
        ("<=1.1", "1.0.1-0", true),
        ("<=1.1", "1.0.0", true),
        ("<=1.1", "1.0.0-0", true),
        ("<=1.1", "0.0.0", true),
        ("<=1.1", "0.0.0-0", true),
        ("<=1.0", "2.0.0", false),
        ("<=1.0", "2.0.0-0", false),
        ("<=1.0", "1.1.0", false),
        ("<=1.0", "1.1.0-0", false),
        ("<=1.0", "1.0.1", true),
        ("<=1.0", "1.0.1-0", true),
        ("<=1.0", "1.0.0", true),
        ("<=1.0", "1.0.0-0", true),
        ("<=1.0", "0.0.0", true),
        ("<=1.0", "0.0.0-0", true),
        ("<=1.1.0", "2.0.0", false),
        ("<=1.1.0", "2.0.0-0", false),
        ("<=1.1.0", "1.1.0", true),
        ("<=1.1.0", "1.1.0-0", true),
        ("<=1.1.0", "1.0.1", true),
        ("<=1.1.0", "1.0.1-0", true),
        ("<=1.1.0", "1.0.0", true),
        ("<=1.1.0", "1.0.0-0", true),
        ("<=1.1.0", "0.0.0", true),
        ("<=1.1.0", "0.0.0-0", true),
        ("<=1.1.0-0", "2.0.0", false),
        ("<=1.1.0-0", "2.0.0-0", false),
        ("<=1.1.0-0", "1.1.0", false),
        ("<=1.1.0-0", "1.1.0-0", true),
        ("<=1.1.0-0", "1.0.1", true),
        ("<=1.1.0-0", "1.0.1-0", true),
        ("<=1.1.0-0", "1.0.0", true),
        ("<=1.1.0-0", "1.0.0-0", true),
        ("<=1.1.0-0", "0.0.0", true),
        ("<=1.1.0-0", "0.0.0-0", true),
        ("<=1.0.1", "2.0.0", false),
        ("<=1.0.1", "2.0.0-0", false),
        ("<=1.0.1", "1.1.0", false),
        ("<=1.0.1", "1.1.0-0", false),
        ("<=1.0.1", "1.0.1", true),
        ("<=1.0.1", "1.0.1-0", true),
        ("<=1.0.1", "1.0.0", true),
        ("<=1.0.1", "1.0.0-0", true),
        ("<=1.0.1", "0.0.0", true),
        ("<=1.0.1", "0.0.0-0", true),
        ("<=1.0.1-0", "2.0.0", false),
        ("<=1.0.1-0", "2.0.0-0", false),
        ("<=1.0.1-0", "1.1.0", false),
        ("<=1.0.1-0", "1.1.0-0", false),
        ("<=1.0.1-0", "1.0.1", false),
        ("<=1.0.1-0", "1.0.1-0", true),
        ("<=1.0.1-0", "1.0.0", true),
        ("<=1.0.1-0", "1.0.0-0", true),
        ("<=1.0.1-0", "0.0.0", true),
        ("<=1.0.1-0", "0.0.0-0", true),
        ("<=1.0.0", "2.0.0", false),
        ("<=1.0.0", "2.0.0-0", false),
        ("<=1.0.0", "1.1.0", false),
        ("<=1.0.0", "1.1.0-0", false),
        ("<=1.0.0", "1.0.1", false),
        ("<=1.0.0", "1.0.1-0", false),
        ("<=1.0.0", "1.0.0", true),
        ("<=1.0.0", "1.0.0-0", true),
        ("<=1.0.0", "0.0.0", true),
        ("<=1.0.0", "0.0.0-0", true),
        ("<=1.0.0-0", "2.0.0", false),
        ("<=1.0.0-0", "2.0.0-0", false),
        ("<=1.0.0-0", "1.1.0", false),
        ("<=1.0.0-0", "1.1.0-0", false),
        ("<=1.0.0-0", "1.0.1", false),
        ("<=1.0.0-0", "1.0.1-0", false),
        ("<=1.0.0-0", "1.0.0", false),
        ("<=1.0.0-0", "1.0.0-0", true),
        ("<=1.0.0-0", "0.0.0", true),
        ("<=1.0.0-0", "0.0.0-0", true),
        (">4.8", "4.9.0-beta", true),
        (">=4.9", "4.9.0-beta", true),
        ("<4.9", "4.9.0-beta", false),
        ("<=4.8", "4.9.0-beta", false),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("comparators", range_text, version_text, *expected);
    }
}

#[test]
fn test_conjunctions_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        (">1.0.0 <2.0.0", "1.0.1", true),
        (">1.0.0 <2.0.0", "2.0.0", false),
        (">1.0.0 <2.0.0", "1.0.0", false),
        (">1 >2", "3.0.0", true),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("conjunctions", range_text, version_text, *expected);
    }
}

#[test]
fn test_disjunctions_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        (">1.0.0 || <1.0.0", "1.0.1", true),
        (">1.0.0 || <1.0.0", "0.0.1", true),
        (">1.0.0 || <1.0.0", "1.0.0", false),
        (">1.0.0 || <1.0.0", "0.0.0", true),
        (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "1.0.0", true),
        (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "2.0.0", false),
        (">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0", "3.0.0", true),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("disjunctions", range_text, version_text, *expected);
    }
}

#[test]
fn test_hyphens_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        ("1.0.0 - 2.0.0", "1.0.0", true),
        ("1.0.0 - 2.0.0", "1.0.1", true),
        ("1.0.0 - 2.0.0", "2.0.0", true),
        ("1.0.0 - 2.0.0", "2.0.1", false),
        ("1.0.0 - 2.0.0", "0.9.9", false),
        ("1.0.0 - 2.0.0", "3.0.0", false),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("hyphens", range_text, version_text, *expected);
    }
}

#[test]
fn test_tildes_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        ("~0", "0.0.0", true),
        ("~0", "0.1.0", true),
        ("~0", "0.1.2", true),
        ("~0", "0.1.9", true),
        ("~0", "1.0.0", false),
        ("~0.1", "0.1.0", true),
        ("~0.1", "0.1.2", true),
        ("~0.1", "0.1.9", true),
        ("~0.1", "0.2.0", false),
        ("~0.1.2", "0.1.2", true),
        ("~0.1.2", "0.1.9", true),
        ("~0.1.2", "0.2.0", false),
        ("~1.0.0", "1.0.0", true),
        ("~1.0.0", "1.0.1", true),
        ("~1", "1.0.0", true),
        ("~1", "1.2.0", true),
        ("~1", "1.2.3", true),
        ("~1", "0.0.0", false),
        ("~1", "2.0.0", false),
        ("~1.2", "1.2.0", true),
        ("~1.2", "1.2.3", true),
        ("~1.2", "1.1.0", false),
        ("~1.2", "1.3.0", false),
        ("~1.2.3", "1.2.3", true),
        ("~1.2.3", "1.2.9", true),
        ("~1.2.3", "1.1.0", false),
        ("~1.2.3", "1.3.0", false),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("tilde", range_text, version_text, *expected);
    }
}

#[test]
fn test_carets_of_version_ranges() {
    let tests: &[(&str, &str, bool)] = &[
        ("^0", "0.0.0", true),
        ("^0", "0.1.0", true),
        ("^0", "0.9.0", true),
        ("^0", "0.1.2", true),
        ("^0", "0.1.9", true),
        ("^0", "1.0.0", false),
        ("^0.1", "0.1.0", true),
        ("^0.1", "0.1.2", true),
        ("^0.1", "0.1.9", true),
        ("^0.1.2", "0.1.2", true),
        ("^0.1.2", "0.1.9", true),
        ("^0.1.2", "0.0.0", false),
        ("^0.1.2", "0.2.0", false),
        ("^0.1.2", "1.0.0", false),
        ("^1", "1.0.0", true),
        ("^1", "1.2.0", true),
        ("^1", "1.2.3", true),
        ("^1", "1.9.0", true),
        ("^1", "0.0.0", false),
        ("^1", "2.0.0", false),
        ("^1.2", "1.2.0", true),
        ("^1.2", "1.2.3", true),
        ("^1.2", "1.9.0", true),
        ("^1.2", "1.1.0", false),
        ("^1.2", "2.0.0", false),
        ("^1.2.3", "1.2.3", true),
        ("^1.2.3", "1.9.0", true),
        ("^1.2.3", "1.2.2", false),
        ("^1.2.3", "2.0.0", false),
    ];
    for (range_text, version_text, expected) in tests {
        assert_range_test("caret", range_text, version_text, *expected);
    }
}
