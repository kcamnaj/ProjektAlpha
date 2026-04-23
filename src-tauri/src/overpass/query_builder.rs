use crate::db::categories::Category;
use crate::error::{AppError, AppResult};
use crate::overpass::tile_splitter::Tile;
use serde_json::Value;

/// osm_tags JSON format:
/// - Outer array = OR-joined alternative rules
/// - Each inner object = AND-joined tag conditions
/// Example: [{"shop":"wholesale","wholesale":"food"}, {"shop":"supermarket"}]
/// matches: (shop=wholesale AND wholesale=food) OR (shop=supermarket)
pub fn build(categories: &[Category], tile: &Tile) -> AppResult<String> {
    if categories.is_empty() {
        return Err(AppError::InvalidInput("no categories".into()));
    }
    let mut out = String::from("[out:json][timeout:25];\n(\n");
    for cat in categories {
        let rules: Value = serde_json::from_str(&cat.osm_tags)
            .map_err(|e| AppError::Internal(format!("bad osm_tags for cat {}: {}", cat.id, e)))?;
        let arr = rules
            .as_array()
            .ok_or_else(|| AppError::Internal("osm_tags not an array".into()))?;
        for rule in arr {
            let obj = rule
                .as_object()
                .ok_or_else(|| AppError::Internal("rule not an object".into()))?;
            let mut conds = String::new();
            for (k, v) in obj {
                let vs = v
                    .as_str()
                    .ok_or_else(|| AppError::Internal("tag value not string".into()))?;
                conds.push_str(&format!("[\"{}\"=\"{}\"]", k, vs));
            }
            out.push_str(&format!(
                "  nwr{}(around:{},{},{});\n",
                conds, tile.radius_m, tile.center_lat, tile.center_lng
            ));
        }
    }
    out.push_str(");\nout center tags;\n");
    Ok(out)
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

    #[test]
    fn single_or_rule() {
        let c = cat(1, r#"[{"shop":"supermarket"}]"#, 75);
        let t = Tile {
            center_lat: 52.0,
            center_lng: 9.0,
            radius_m: 50_000,
        };
        let q = build(&[c], &t).unwrap();
        assert!(q.contains("[\"shop\"=\"supermarket\"]"));
        assert!(q.contains("(around:50000,52,9)"));
        assert!(q.starts_with("[out:json]"));
        assert!(q.contains("out center tags"));
    }

    #[test]
    fn multi_tag_and_within_one_rule() {
        let c = cat(1, r#"[{"shop":"wholesale","wholesale":"food"}]"#, 90);
        let t = Tile {
            center_lat: 50.0,
            center_lng: 8.0,
            radius_m: 25_000,
        };
        let q = build(&[c], &t).unwrap();
        assert!(
            q.contains("[\"shop\"=\"wholesale\"][\"wholesale\"=\"food\"]")
                || q.contains("[\"wholesale\"=\"food\"][\"shop\"=\"wholesale\"]")
        );
    }

    #[test]
    fn empty_categories_errors() {
        let t = Tile {
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 1000,
        };
        assert!(build(&[], &t).is_err());
    }

    #[test]
    fn malformed_tags_errors() {
        let c = cat(1, r#"not json"#, 50);
        let t = Tile {
            center_lat: 0.0,
            center_lng: 0.0,
            radius_m: 1000,
        };
        assert!(build(&[c], &t).is_err());
    }
}
