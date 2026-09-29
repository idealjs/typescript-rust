#![allow(unused_imports)]

use crate::checker::inference::*;

use crate::checker::string_surrogate::{advance_js_code_point, combine_surrogate_pairs};

enum PartMatch {
    Single(String),
    Multi(Vec<String>, usize, usize),
}

impl Checker {
    pub(crate) fn infer_to_template_literal_type(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &TemplateLiteralTypeData,
    ) {
        let matches = self.infer_types_from_template_literal_type(source, target);
        let has_matches = matches.as_ref().is_some_and(|m| !m.is_empty());
        if has_matches || target.texts.iter().all(|t| t.is_empty()) {
            let matches = matches.unwrap_or_default();
            for (i, target_type) in target.types.iter().enumerate() {
                let source_type = if has_matches {
                    Arc::clone(&matches[i])
                } else {
                    self.never_type()
                };
                self.infer_from_types(state, &source_type, target_type);
            }
        }
    }

    pub fn infer_types_from_template_literal_type(
        &mut self,
        source: &Arc<Type>,
        target: &TemplateLiteralTypeData,
    ) -> Option<Vec<Arc<Type>>> {
        if source.flags.contains(TypeFlags::StringLiteral) {
            let value = self.string_literal_value_of(source)?;
            return self.infer_from_literal_parts_to_template_literal(&[value], &[], target);
        }
        if source.flags.contains(TypeFlags::TemplateLiteral)
            && let TypeData::TemplateLiteral(st) = &source.data
        {
            if st.texts == target.texts {
                let mut result = Vec::with_capacity(st.types.len());
                for (s, t) in st.types.iter().zip(target.types.iter()) {
                    let cs = self.get_base_constraint_or_type(s);
                    let ct = self.get_base_constraint_or_type(t);
                    let keep = self.is_type_assignable_to(&cs, &ct);
                    result.push(if keep {
                        Arc::clone(s)
                    } else {
                        self.get_string_like_type_for_type(s)
                    });
                }
                return Some(result);
            }
            return self.infer_from_literal_parts_to_template_literal(&st.texts, &st.types, target);
        }
        None
    }

    pub fn get_string_like_type_for_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TypeFlags::Any | TYPE_FLAGS_STRING_LIKE) {
            return Arc::clone(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts: vec![String::new(), String::new()],
                types: vec![Arc::clone(t)],
            }),
        ))
    }

    pub fn template_literal_types_definitely_unrelated(
        &mut self,
        source: &TemplateLiteralTypeData,
        target: &TemplateLiteralTypeData,
    ) -> bool {
        let source_start = &source.texts[0];
        let target_start = &target.texts[0];
        let source_end = &source.texts[source.texts.len() - 1];
        let target_end = &target.texts[target.texts.len() - 1];
        let start_len = source_start.len().min(target_start.len());
        let end_len = source_end.len().min(target_end.len());
        source_start[..start_len] != target_start[..start_len]
            || source_end[source_end.len() - end_len..] != target_end[target_end.len() - end_len..]
    }

    fn string_literal_value_of(&self, t: &Arc<Type>) -> Option<String> {
        if let TypeData::Literal(lit) = &t.data
            && let LiteralValue::String(s) = &lit.value
        {
            return Some(s.clone());
        }
        None
    }

    pub(crate) fn infer_from_literal_parts_to_template_literal(
        &mut self,
        source_texts: &[String],
        source_types: &[Arc<Type>],
        target: &TemplateLiteralTypeData,
    ) -> Option<Vec<Arc<Type>>> {
        let parts = self.match_literal_like_pattern(source_texts, target)?;
        let mut matches = Vec::with_capacity(parts.len());
        for part in parts {
            let match_type = match part {
                PartMatch::Single(text) => {
                    self.get_string_literal_type(&combine_surrogate_pairs(&text))
                }
                PartMatch::Multi(texts, seg, s) => Arc::new(Type::new(
                    TypeFlags::TemplateLiteral,
                    TypeData::TemplateLiteral(TemplateLiteralTypeData {
                        constrained: ConstrainedTypeData::default(),
                        texts,
                        types: source_types[seg..s].to_vec(),
                    }),
                )),
            };
            matches.push(match_type);
        }
        Some(matches)
    }

    fn match_literal_like_pattern(
        &mut self,
        source_texts: &[String],
        target: &TemplateLiteralTypeData,
    ) -> Option<Vec<PartMatch>> {
        let last_source_index = source_texts.len() - 1;
        let source_start_text = source_texts[0].as_str();
        let source_end_text = source_texts[last_source_index].as_str();
        let target_texts = &target.texts;
        let last_target_index = target_texts.len() - 1;
        let target_start_text = target_texts[0].as_str();
        let target_end_text = target_texts[last_target_index].as_str();
        if last_source_index == 0
            && source_start_text.len() < target_start_text.len() + target_end_text.len()
            || !source_start_text.starts_with(target_start_text)
            || !source_end_text.ends_with(target_end_text)
        {
            return None;
        }
        let remaining_end_text =
            source_end_text[..source_end_text.len() - target_end_text.len()].to_string();
        let get_source_text = |index: usize| -> &str {
            if index < last_source_index {
                source_texts[index].as_str()
            } else {
                remaining_end_text.as_str()
            }
        };
        let mut seg = 0usize;
        let mut pos = target_start_text.len();
        let mut matches: Vec<PartMatch> = Vec::new();
        let mut add_match = |seg: &mut usize, pos: &mut usize, s: usize, p: usize| {
            let match_part = if s == *seg {
                PartMatch::Single(get_source_text(s)[*pos..p].to_string())
            } else {
                let mut match_texts = Vec::with_capacity(s - *seg + 1);
                match_texts.push(source_texts[*seg][*pos..].to_string());
                for text in &source_texts[*seg + 1..s] {
                    match_texts.push(text.clone());
                }
                match_texts.push(get_source_text(s)[..p].to_string());
                PartMatch::Multi(match_texts, *seg, s)
            };
            matches.push(match_part);
            *seg = s;
            *pos = p;
        };
        for i in 1..last_target_index {
            let delim = target_texts[i].as_str();
            if !delim.is_empty() {
                let mut s = seg;
                let mut p = pos;
                loop {
                    let text = get_source_text(s);
                    match text[p..].find(delim) {
                        Some(d) => {
                            p += d;
                            break;
                        }
                        None => {
                            s += 1;
                            if s == source_texts.len() {
                                return None;
                            }
                            p = 0;
                        }
                    }
                }
                add_match(&mut seg, &mut pos, s, p);
                pos += delim.len();
            } else {
                let source_text = get_source_text(seg);
                if pos < source_text.len() {
                    let next_pos = pos + advance_js_code_point(source_text, pos);
                    let cur_seg = seg;
                    add_match(&mut seg, &mut pos, cur_seg, next_pos);
                } else if seg < last_source_index {
                    let next_seg = seg + 1;
                    add_match(&mut seg, &mut pos, next_seg, 0);
                } else {
                    return None;
                }
            }
        }
        add_match(
            &mut seg,
            &mut pos,
            last_source_index,
            get_source_text(last_source_index).len(),
        );
        Some(matches)
    }
}
