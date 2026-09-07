#![allow(unused_imports)]

use super::*;

pub fn get_common_parents(
    paths: &[String],
    min_components: usize,
    options: &ComparePathsOptions,
) -> (Vec<String>, std::collections::HashSet<String>) {
    if min_components < 1 {
        panic!("minComponents must be at least 1");
    }
    if paths.is_empty() {
        return (vec![], std::collections::HashSet::new());
    }
    if paths.len() == 1 {
        let components =
            reduce_path_components(&get_path_components(&paths[0], &options.current_directory));
        if components.len() < min_components {
            let mut ignored = std::collections::HashSet::new();
            ignored.insert(paths[0].clone());
            return (vec![], ignored);
        }
        return (vec![paths[0].clone()], std::collections::HashSet::new());
    }

    let mut ignored = std::collections::HashSet::new();
    let mut path_components: Vec<Vec<String>> = Vec::new();
    for path in paths {
        let components =
            reduce_path_components(&get_path_components(path, &options.current_directory));
        if components.len() < min_components {
            ignored.insert(path.clone());
        } else {
            path_components.push(components);
        }
    }

    let results = get_common_parents_worker(&path_components, min_components, options);
    let result_paths: Vec<String> = results
        .iter()
        .map(|comps| get_path_from_path_components(comps))
        .collect();

    (result_paths, ignored)
}

pub(crate) fn get_common_parents_worker(
    component_groups: &[Vec<String>],
    min_components: usize,
    options: &ComparePathsOptions,
) -> Vec<Vec<String>> {
    if component_groups.is_empty() {
        return vec![];
    }

    let max_depth = component_groups.iter().map(|g| g.len()).min().unwrap_or(0);
    let equality = options.equality_comparer();

    for last_common_index in 0..max_depth {
        let candidate = &component_groups[0][last_common_index];
        for j in 1..component_groups.len() {
            let comps = &component_groups[j];
            if !equality(candidate, &comps[last_common_index]) {
                if last_common_index < min_components {
                    let mut ordered_groups: Vec<String> = Vec::new();
                    let mut new_groups: std::collections::HashMap<
                        String,
                        (Vec<String>, Vec<Vec<String>>),
                    > = std::collections::HashMap::new();

                    for g in component_groups {
                        let key = to_path(
                            &g[last_common_index],
                            &options.current_directory,
                            options.use_case_sensitive_file_names,
                        )
                        .to_string();
                        if !new_groups.contains_key(&key) {
                            ordered_groups.push(key.clone());
                        }
                        let entry = new_groups.entry(key).or_insert_with(|| (vec![], vec![]));
                        if entry.0.is_empty() {
                            entry.0 = g[..=last_common_index].to_vec();
                        }
                        entry.1.push(g[last_common_index + 1..].to_vec());
                    }

                    ordered_groups.sort();
                    let mut result: Vec<Vec<String>> = vec![];
                    for key in &ordered_groups {
                        let group = &new_groups[key];
                        let sub_results = get_common_parents_worker(
                            &group.1,
                            min_components.saturating_sub(last_common_index + 1),
                            options,
                        );
                        for sr in &sub_results {
                            let mut combined = group.0.clone();
                            combined.extend(sr.iter().cloned());
                            result.push(combined);
                        }
                    }
                    return result;
                }
                return vec![component_groups[0][..last_common_index].to_vec()];
            }
        }
    }

    vec![component_groups[0][..max_depth].to_vec()]
}
