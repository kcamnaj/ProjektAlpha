use crate::db::categories::Category;
use std::collections::HashMap;

/// Find first category whose osm_tags rule matches the given tag map.
/// AND inside one rule object, OR across rule objects.
pub fn match_category<'a>(
    tags: &HashMap<String, String>,
    categories: &'a [Category],
) -> Option<&'a Category> {
    for cat in categories {
        let rules: serde_json::Value = match serde_json::from_str(&cat.osm_tags) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let arr = match rules.as_array() {
            Some(a) => a,
            None => continue,
        };
        for rule in arr {
            let obj = match rule.as_object() {
                Some(o) => o,
                None => continue,
            };
            if obj.iter().all(|(k, v)| {
                tags.get(k)
                    .map(|tv| Some(tv.as_str()) == v.as_str())
                    .unwrap_or(false)
            }) {
                return Some(cat);
            }
        }
    }
    None
}

pub fn score_for_category(cat: &Category) -> i64 {
    cat.probability_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(id: i64, tags: &str, w: i64) -> Category {
        Category {
            id,
            name_de: format!("c{id}"),
            osm_tags: tags.into(),
            probability_weight: w,
            enabled: true,
            color: "#000".into(),
        }
    }

    fn tagmap(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn matches_single_tag() {
        let cs = vec![cat(1, r#"[{"shop":"supermarket"}]"#, 75)];
        assert!(match_category(&tagmap(&[("shop", "supermarket")]), &cs).is_some());
    }

    #[test]
    fn requires_all_tags_in_rule() {
        let cs = vec![cat(1, r#"[{"shop":"wholesale","wholesale":"food"}]"#, 90)];
        assert!(match_category(&tagmap(&[("shop", "wholesale")]), &cs).is_none());
        assert!(match_category(
            &tagmap(&[("shop", "wholesale"), ("wholesale", "food")]),
            &cs
        )
        .is_some());
    }

    #[test]
    fn or_alternative_rules() {
        let cs = vec![cat(1, r#"[{"shop":"a"},{"shop":"b"}]"#, 50)];
        assert!(match_category(&tagmap(&[("shop", "a")]), &cs).is_some());
        assert!(match_category(&tagmap(&[("shop", "b")]), &cs).is_some());
        assert!(match_category(&tagmap(&[("shop", "c")]), &cs).is_none());
    }

    #[test]
    fn first_matching_category_wins() {
        let cs = vec![
            cat(1, r#"[{"shop":"supermarket"}]"#, 75),
            cat(2, r#"[{"shop":"supermarket"}]"#, 99),
        ];
        let m = match_category(&tagmap(&[("shop", "supermarket")]), &cs).unwrap();
        assert_eq!(m.id, 1);
    }
}
