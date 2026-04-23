#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_m: u32,
}

pub fn split(center_lat: f64, center_lng: f64, radius_km: u32) -> Vec<Tile> {
    if radius_km <= 50 {
        return vec![Tile {
            center_lat,
            center_lng,
            radius_m: radius_km * 1000,
        }];
    }
    if radius_km <= 150 {
        let half = radius_km as f64 / 2.0;
        let lat_off = km_to_lat_deg(half);
        let lng_off = km_to_lng_deg(half, center_lat);
        let r_m = (half * 1000.0) as u32;
        return vec![
            Tile {
                center_lat: center_lat + lat_off,
                center_lng: center_lng + lng_off,
                radius_m: r_m,
            },
            Tile {
                center_lat: center_lat + lat_off,
                center_lng: center_lng - lng_off,
                radius_m: r_m,
            },
            Tile {
                center_lat: center_lat - lat_off,
                center_lng: center_lng + lng_off,
                radius_m: r_m,
            },
            Tile {
                center_lat: center_lat - lat_off,
                center_lng: center_lng - lng_off,
                radius_m: r_m,
            },
        ];
    }
    // > 150 km: 50 km grid, circle-clip
    let cell_km = 50.0_f64;
    let n = (2.0 * radius_km as f64 / cell_km).ceil() as i32;
    let half_n = n / 2;
    let lat_step = km_to_lat_deg(cell_km);

    let mut tiles = vec![];
    for i in -half_n..=half_n {
        for j in -half_n..=half_n {
            let lat = center_lat + (i as f64) * lat_step;
            let lng_step = km_to_lng_deg(cell_km, lat);
            let lng = center_lng + (j as f64) * lng_step;
            let dist_km = haversine_km(center_lat, center_lng, lat, lng);
            if dist_km <= radius_km as f64 + cell_km / 2.0 {
                tiles.push(Tile {
                    center_lat: lat,
                    center_lng: lng,
                    radius_m: 25_000,
                });
            }
        }
    }
    tiles
}

fn km_to_lat_deg(km: f64) -> f64 {
    km / 111.0
}
fn km_to_lng_deg(km: f64, at_lat: f64) -> f64 {
    km / (111.0 * at_lat.to_radians().cos())
}

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_radius_returns_single_tile() {
        let tiles = split(52.37, 9.73, 50);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].radius_m, 50_000);
    }

    #[test]
    fn medium_radius_returns_four_quadrants() {
        let tiles = split(52.37, 9.73, 100);
        assert_eq!(tiles.len(), 4);
        assert!(tiles.iter().all(|t| t.radius_m == 50_000));
    }

    #[test]
    fn large_radius_returns_grid_within_circle() {
        let tiles = split(52.37, 9.73, 300);
        assert!(tiles.len() > 4, "got {}", tiles.len());
        for t in &tiles {
            let d = haversine_km(52.37, 9.73, t.center_lat, t.center_lng);
            assert!(d <= 300.0 + 25.0, "tile {:?} too far ({} km)", t, d);
        }
    }

    #[test]
    fn large_radius_covers_center_and_edges() {
        let tiles = split(52.37, 9.73, 200);
        let any_near_center = tiles
            .iter()
            .any(|t| haversine_km(52.37, 9.73, t.center_lat, t.center_lng) < 30.0);
        assert!(any_near_center, "no tile near center");
        let any_far = tiles
            .iter()
            .any(|t| haversine_km(52.37, 9.73, t.center_lat, t.center_lng) > 150.0);
        assert!(any_far, "no outer-ring tile");
    }
}
