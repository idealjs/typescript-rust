pub const CHECKER_ASSOCIATION_TEXT_WEIGHT_DIVISOR: usize = 100;
pub const CHECKER_ASSOCIATION_SOURCE_FILE_WEIGHT_MULTIPLIER: usize = 4;
pub const CHECKER_ASSOCIATION_BALANCE_PENALTY_MULTIPLIER: usize = 16;
pub const CHECKER_ASSOCIATION_PRIORITIZED_SOURCE_PENALTY: usize = 12;
pub const CHECKER_ASSOCIATION_STRONG_BALANCE_MIN_CHECKER_COUNT: usize = 4;

pub struct CheckerAssociationPolicy {
    pub prioritize_source_files: bool,
    pub source_file_weight_multiplier: usize,
    pub balance_penalty_multiplier: usize,
}

pub fn get_checker_association_policy(
    total_weight: usize,
    declaration_weight: usize,
    checker_count: usize,
) -> CheckerAssociationPolicy { ::tsox_core::fntrace::enter("get_checker_association_policy"); 
    if should_prioritize_source_files(total_weight, declaration_weight, checker_count) {
        return CheckerAssociationPolicy {
            prioritize_source_files: true,
            source_file_weight_multiplier: 1,
            balance_penalty_multiplier: CHECKER_ASSOCIATION_PRIORITIZED_SOURCE_PENALTY,
        };
    }
    if checker_count >= CHECKER_ASSOCIATION_STRONG_BALANCE_MIN_CHECKER_COUNT {
        return CheckerAssociationPolicy {
            prioritize_source_files: false,
            source_file_weight_multiplier: CHECKER_ASSOCIATION_SOURCE_FILE_WEIGHT_MULTIPLIER,
            balance_penalty_multiplier: CHECKER_ASSOCIATION_BALANCE_PENALTY_MULTIPLIER,
        };
    }
    CheckerAssociationPolicy {
        prioritize_source_files: false,
        source_file_weight_multiplier: 1,
        balance_penalty_multiplier: 1,
    }
}

pub fn get_checker_associations_in_order(
    file_weights: &[usize],
    adjacent_files: &[Vec<usize>],
    file_order: Option<&[usize]>,
    checker_count: usize,
    penalty_multiplier: usize,
) -> Vec<usize> { ::tsox_core::fntrace::enter("get_checker_associations_in_order"); 
    if file_weights.is_empty() {
        return Vec::new();
    }

    let mut total_weight = 0usize;
    let mut max_file_weight = 0usize;
    let mut edge_count = 0usize;
    for (i, weight) in file_weights.iter().enumerate() {
        total_weight += weight;
        max_file_weight = max_file_weight.max(*weight);
        edge_count += adjacent_files[i].len();
    }

    let mut associations: Vec<usize> = vec![usize::MAX; file_weights.len()];
    let mut checker_weights = vec![0usize; checker_count];
    let average_checker_weight = total_weight.div_ceil(checker_count);
    let max_checker_weight =
        max_file_weight.max(average_checker_weight + average_checker_weight / 100);
    let total_weight_float = total_weight as f64;
    let alpha = penalty_multiplier as f64
        * (edge_count / 2) as f64
        * (checker_count as f64).sqrt()
        / (total_weight_float * total_weight_float.sqrt());
    let mut neighbor_counts = vec![0usize; checker_count];

    for position in 0..file_weights.len() {
        let file_index = match file_order {
            Some(order) => order[position],
            None => position,
        };

        for count in neighbor_counts.iter_mut() {
            *count = 0;
        }
        for &adjacent_file in &adjacent_files[file_index] {
            let checker_index = associations[adjacent_file];
            if checker_index != usize::MAX {
                neighbor_counts[checker_index] += 1;
            }
        }

        let mut best_checker: i64 = -1;
        let mut best_score = f64::NEG_INFINITY;
        for (checker_index, &checker_weight) in checker_weights.iter().enumerate() {
            if checker_weight + file_weights[file_index] > max_checker_weight {
                continue;
            }
            let old_weight = checker_weight as f64;
            let new_weight = (checker_weight + file_weights[file_index]) as f64;
            let penalty = alpha * (new_weight * new_weight.sqrt() - old_weight * old_weight.sqrt());
            let score = neighbor_counts[checker_index] as f64 - penalty;
            if score > best_score
                || (score == best_score
                    && (best_checker < 0 || checker_weight < checker_weights[best_checker as usize]))
            {
                best_checker = checker_index as i64;
                best_score = score;
            }
        }
        let best_checker = if best_checker < 0 {
            let mut least = 0usize;
            for (checker_index, &checker_weight) in checker_weights.iter().enumerate().skip(1) {
                if checker_weight < checker_weights[least] {
                    least = checker_index;
                }
            }
            least
        } else {
            best_checker as usize
        };
        associations[file_index] = best_checker;
        checker_weights[best_checker] += file_weights[file_index];
    }
    associations
}

pub fn get_checker_association_order(
    file_weights: &[usize],
    is_declaration_file: &[bool],
    prioritize_source_files: bool,
) -> Option<Vec<usize>> { ::tsox_core::fntrace::enter("get_checker_association_order"); 
    if !prioritize_source_files {
        return None;
    }
    let mut file_order: Vec<usize> = (0..file_weights.len()).collect();
    file_order.sort_by(|&left, &right| {
        is_declaration_file[left]
            .cmp(&is_declaration_file[right])
            .then_with(|| file_weights[right].cmp(&file_weights[left]))
            .then_with(|| left.cmp(&right))
    });
    Some(file_order)
}

pub fn get_checker_association_base_weight(node_count: usize, text_length: usize) -> usize { ::tsox_core::fntrace::enter("get_checker_association_base_weight"); 
    (node_count + text_length / CHECKER_ASSOCIATION_TEXT_WEIGHT_DIVISOR).max(1)
}

pub fn should_prioritize_source_files(
    total_weight: usize,
    declaration_weight: usize,
    checker_count: usize,
) -> bool { ::tsox_core::fntrace::enter("should_prioritize_source_files"); 
    declaration_weight * checker_count * 2 <= total_weight
}

pub fn get_checker_association_weights(
    base_weights: &[usize],
    import_counts: &[usize],
) -> Vec<usize> { ::tsox_core::fntrace::enter("get_checker_association_weights"); 
    let mut total_base_weight = 0usize;
    let mut total_imports = 0usize;
    for (i, base_weight) in base_weights.iter().enumerate() {
        total_base_weight += base_weight;
        total_imports += import_counts[i];
    }
    let mut import_weight = 0usize;
    if total_imports > 0 {
        import_weight = (total_base_weight / total_imports).max(1);
    }
    base_weights
        .iter()
        .zip(import_counts)
        .map(|(base_weight, import_count)| base_weight + import_count * import_weight)
        .collect()
}

