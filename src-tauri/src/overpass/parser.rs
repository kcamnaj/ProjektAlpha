use crate::db::categories::Category;
use crate::db::companies::NewCompany;
use crate::error::AppResult;
use crate::overpass::scoring::{match_category, score_for_category};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Response {
    elements: Vec<Element>,
}

#[derive(Deserialize)]
struct Element {
    #[serde(rename = "type")]
    kind: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<Center>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Deserialize)]
struct Center {
    lat: f64,
    lon: f64,
}

pub fn parse(json: &str, categories: &[Category]) -> AppResult<Vec<NewCompany>> {
    let resp: Response = serde_json::from_str(json)?;
    let mut out = Vec::new();
    for el in resp.elements {
        let (lat, lng) = match (el.lat, el.lon, &el.center) {
            (Some(la), Some(lo), _) => (la, lo),
            (_, _, Some(c)) => (c.lat, c.lon),
            _ => continue,
        };

        let cat = match match_category(&el.tags, categories) {
            Some(c) => c,
            None => continue, // unknown industry → skip
        };

        let osm_id = format!("{}/{}", el.kind, el.id);
        let name = el.tags.get("name").cloned().unwrap_or_else(|| {
            let plz = el
                .tags
                .get("addr:postcode")
                .map(String::as_str)
                .unwrap_or("?");
            let city = el.tags.get("addr:city").map(String::as_str).unwrap_or("");
            format!("Unbenannt ({} {})", plz, city).trim().to_string()
        });

        let street = el.tags.get("addr:street").map(|s| {
            let nr = el
                .tags
                .get("addr:housenumber")
                .map(String::as_str)
                .unwrap_or("");
            format!("{} {}", s, nr).trim().to_string()
        });

        out.push(NewCompany {
            osm_id: Some(osm_id),
            name,
            street,
            postal_code: el.tags.get("addr:postcode").cloned(),
            city: el.tags.get("addr:city").cloned(),
            country: el
                .tags
                .get("addr:country")
                .cloned()
                .unwrap_or_else(|| "DE".into()),
            lat,
            lng,
            phone: el
                .tags
                .get("phone")
                .cloned()
                .or_else(|| el.tags.get("contact:phone").cloned()),
            email: el
                .tags
                .get("email")
                .cloned()
                .or_else(|| el.tags.get("contact:email").cloned()),
            website: el
                .tags
                .get("website")
                .cloned()
                .or_else(|| el.tags.get("contact:website").cloned()),
            industry_category_id: Some(cat.id),
            size_estimate: None,
            probability_score: score_for_category(cat),
            source: "osm".into(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds() -> Vec<Category> {
        vec![
            Category {
                id: 1,
                name_de: "LM-GH".into(),
                osm_tags: r#"[{"shop":"wholesale","wholesale":"food"}]"#.into(),
                probability_weight: 90,
                enabled: true,
                color: "#000".into(),
            },
            Category {
                id: 2,
                name_de: "Lager".into(),
                osm_tags: r#"[{"industrial":"warehouse"}]"#.into(),
                probability_weight: 85,
                enabled: true,
                color: "#000".into(),
            },
        ]
    }

    #[test]
    fn parses_node_with_full_address() {
        let json = include_str!("../../tests/fixtures/overpass_simple.json");
        let r = parse(json, &seeds()).unwrap();
        assert_eq!(r.len(), 1);
        let c = &r[0];
        assert_eq!(c.name, "Müller Logistik GmbH");
        assert_eq!(c.osm_id.as_deref(), Some("node/12345"));
        assert_eq!(c.street.as_deref(), Some("Industriestr. 12"));
        assert_eq!(c.postal_code.as_deref(), Some("30659"));
        assert_eq!(c.industry_category_id, Some(1));
        assert_eq!(c.probability_score, 90);
    }

    #[test]
    fn parses_way_via_center_and_skips_unknown() {
        let json = include_str!("../../tests/fixtures/overpass_with_polygons.json");
        let r = parse(json, &seeds()).unwrap();
        assert_eq!(r.len(), 1, "polygon match wins, unknown shop skipped");
        assert_eq!(r[0].osm_id.as_deref(), Some("way/99999"));
        assert_eq!(r[0].lat, 53.0);
        assert_eq!(r[0].industry_category_id, Some(2));
    }
}
