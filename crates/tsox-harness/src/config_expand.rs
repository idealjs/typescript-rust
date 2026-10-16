pub const VARY_BY: &[&str] = &[
    "jsx", "module", "moduledetection", "moduleresolution", "newline", "target",
    "allowarbitraryextensions", "allowimportingtsextensions", "allowjs",
    "allowsyntheticdefaultimports", "allowumdglobalaccess", "allowunreachablecode",
    "allowunusedlabels", "alwaysstrict", "assumechangesonlyaffectdirectdependencies",
    "checkjs", "composite", "declaration", "declarationmap", "deduplicatepackages",
    "disablesizelimit", "downleveliteration", "emitbom", "emitdeclarationonly",
    "emitdecoratormetadata", "erasablesyntaxonly", "esmoduleinterop",
    "exactoptionalpropertytypes", "experimentaldecorators",
    "forceconsistentcasinginfilenames", "importhelpers", "inlinesourcemap",
    "inlinesources", "isolateddeclarations", "isolatedmodules", "libreplacement",
    "noemit", "noemithelpers", "noemitonerror", "noerrortruncation",
    "nofallthroughcasesinswitch", "noimplicitany", "noimplicitoverride",
    "noimplicitreturns", "noimplicitthis", "nolib", "nopropertyaccessfromindexsignature",
    "noresolve", "nouncheckedindexedaccess", "nouncheckedsideeffectimports",
    "preserveconstenums", "removecomments",
    "resolvejsonmodule", "resolvepackagejsonexports", "resolvepackagejsonimports",
    "rewriterelativeimportextensions", "skipdefaultlibcheck", "skiplibcheck",
    "sourcemap", "stabletypeordering", "strict", "strictbindcallapply",
    "strictbuiltiniteratorreturn", "strictfunctiontypes", "strictnullchecks",
    "strictpropertyinitialization", "stripinternal", "usedefineforclassfields",
    "useunknownincatchvariables", "verbatimmodulesyntax",
];

fn all_enum_values(option: &str) -> &'static [&'static str] {
    match option {
        "target" => &[
            "es5", "es6", "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021",
            "es2022", "es2023", "es2024", "es2025", "esnext",
        ],
        "module" => &[
            "commonjs", "amd", "system", "umd", "es6", "es2015", "es2020", "es2022", "esnext",
            "node16", "node18", "node20", "nodenext", "preserve",
        ],
        "moduleresolution" => &[
            "node16", "nodenext", "bundler", "classic", "node", "node10",
        ],
        "jsx" => &["preserve", "react-native", "react-jsx", "react-jsxdev", "react"],
        "moduledetection" => &["auto", "legacy", "force"],
        "newline" => &["crlf", "lf"],
        _ => &[],
    }
}

fn canonical_value(option: &str, raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    match option {
        "target" => match lower.as_str() {
            "es6" => "es2015".to_string(),
            other => other.to_string(),
        },
        "module" => match lower.as_str() {
            "es6" => "es2015".to_string(),
            other => other.to_string(),
        },
        "moduleresolution" => match lower.as_str() {
            "node" => "node10".to_string(),
            other => other.to_string(),
        },

        _ if VARY_BY.contains(&option) && !all_enum_values(option).is_empty() => lower,
        _ if VARY_BY.contains(&option) => match lower.as_str() {
            "true" => "true".to_string(),
            _ => "false".to_string(),
        },
        _ => lower,
    }
}

fn split_option_values(value: &str, option: &str) -> Option<Vec<String>> {
    let mut star = false;
    let mut includes: Vec<String> = Vec::new();
    let mut excludes: Vec<String> = Vec::new();
    for part in value.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        if s == "*" {
            star = true;
        } else if let Some(rest) = s.strip_prefix('-').or_else(|| s.strip_prefix('!')) {
            excludes.push(rest.trim().to_string());
        } else {
            includes.push(s.to_string());
        }
    }
    if includes.is_empty() && !star && excludes.is_empty() {
        return None;
    }
    if star {
        if all_enum_values(option).is_empty() {
            includes.push("true".to_string());
            includes.push("false".to_string());
        } else {
            includes.extend(all_enum_values(option).iter().map(|s| s.to_string()));
        }
    }

    let mut seen: Vec<String> = Vec::new();
    let mut raws: Vec<String> = Vec::new();
    for raw in includes {
        let canon = canonical_value(option, &raw);
        if !seen.contains(&canon) {
            seen.push(canon);
            raws.push(raw);
        }
    }
    for ex in excludes {
        let canon = canonical_value(option, &ex);
        if let Some(pos) = seen.iter().position(|c| *c == canon) {
            seen.remove(pos);
            raws.remove(pos);
        }
    }
    Some(raws)
}

// tsgo 已移除的选项值（TS5108: alwaysStrict=false），对应变体在
// 再生参考基线中不存在，枚举时按无效变体剔除
fn is_removed_option_value(option: &str, raw: &str) -> bool {
    matches!(
        (option, canonical_value(option, raw).as_str()),
        ("alwaysstrict", "false")
    )
}

pub fn compute_configurations(
    settings: &std::collections::HashMap<String, String>,
) -> Result<Vec<(String, std::collections::HashMap<String, String>)>, String> {
    let mut varying: Vec<(String, Vec<String>)> = Vec::new();
    let mut base = settings.clone();
    for key in settings.keys() {
        if !VARY_BY.contains(&key.as_str()) {
            continue;
        }
        let value = &settings[key];
        if !value.contains(',') && value.trim() != "*" {

            continue;
        }
        let Some(values) = split_option_values(value, key) else {
            continue;
        };
        // 已移除的选项值（alwaysStrict=false）不参与枚举；原注解多值时
        // 保持变体结构（剔除后单值仍按变体生成，不折叠进 base，否则产物
        // 文件名失去变体后缀与参考基线错位）
        let raw_len = values.len();
        let values: Vec<String> = values
            .into_iter()
            .filter(|v| !is_removed_option_value(key, v))
            .collect();
        if values.is_empty() {
            continue;
        }
        if raw_len <= 1 {
            if let [only] = values.as_slice() {
                base.insert(key.clone(), only.clone());
            }
            continue;
        }
        varying.push((key.clone(), values));
    }

    if varying.is_empty() {
        return Ok(vec![(String::new(), base)]);
    }

    let mut count = 1usize;
    for (_, values) in &varying {
        count *= values.len();
        if count > 25 {
            return Err(format!("too many option variations ({count})"));
        }
    }

    let mut configs: Vec<std::collections::HashMap<String, String>> =
        vec![std::collections::HashMap::new()];
    for (key, values) in &varying {
        let mut next = Vec::with_capacity(configs.len() * values.len());
        for config in &configs {
            for value in values {
                let mut c = config.clone();
                c.insert(key.clone(), value.clone());
                next.push(c);
            }
        }
        configs = next;
    }

    let mut out = Vec::with_capacity(configs.len());
    for config in configs {

        let mut parts: Vec<String> = config
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_ascii_lowercase(), v.to_ascii_lowercase()))
            .collect();
        parts.sort();
        let suffix = parts.join(",");
        let mut merged = base.clone();
        for (k, v) in config {
            merged.insert(k, v);
        }
        out.push((suffix, merged));
    }
    Ok(out)
}
