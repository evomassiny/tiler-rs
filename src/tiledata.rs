use tile::{Tile,Bbox};
use std::cmp::min;
use std::f32;
use utils::search_closest_idx;

pub const TILE_SIZE: usize = 256;

/// Holds data and provides methods to regrid data into a 256 x 256 grid.
#[derive(Debug)]
pub struct TileData {
    /// Must be expressed in meters, in ascending order
    pub lat: Vec<f64>,
    /// Must be expressed in meters, in ascending order
    pub lon: Vec<f64>,
    /// Values must be a flattened array (lat, lon)
    pub values: Vec<f32>,
    pub bbox: Bbox,
    pub tile: Tile,
}
impl TileData {

    /**
     * regrid self.values into a TILE_SIZE x TILE_SIZE grid.
     */
    pub fn to_tile_grid(&self) -> [[f32; TILE_SIZE]; TILE_SIZE] {

        // fetch nearest latitudes indices
        let mut lat_ids: [usize; TILE_SIZE] = [0; TILE_SIZE];
        let lat_inc: f64 = (self.bbox.north -self.bbox.south) / (TILE_SIZE as f64);

        let lats: Vec<f64> = (0..TILE_SIZE).map(|i| {
            self.bbox.south + lat_inc * (0.5 + i as f64)
        }).collect();
        for (i, lat) in lats.iter().enumerate() {
            lat_ids[i] = search_closest_idx(&self.lat, lat).unwrap();
        }

        // fetch nearest longitude indices
        let mut lon_ids: [usize; TILE_SIZE] = [0; TILE_SIZE];
        let lon_inc: f64 = (self.bbox.east - self.bbox.west) / (TILE_SIZE as f64);

        let lons: Vec<f64> = (0..TILE_SIZE).map(|i| {
            self.bbox.west + lon_inc * (0.5 + i as f64)
        }).collect();
        for (i, lon) in lons.iter().enumerate() {
            lon_ids[i] = search_closest_idx(&self.lon, lon).unwrap();
        }

        // pick values using precomputed indices
        let mut values: [[f32; TILE_SIZE]; TILE_SIZE] = [
            [f32::NAN; TILE_SIZE];
            TILE_SIZE
        ];
        if self.values.len() > TILE_SIZE * TILE_SIZE {
            for i_lat in 0..TILE_SIZE {
                for i_lon in 0..TILE_SIZE {
                    values[i_lat][i_lon] = self.value_at(lat_ids[i_lat], lon_ids[i_lon]);
                }
            }

        } else {
            for (i_lat, lat) in lats.iter().enumerate() {
                for (i_lon, lon) in lons.iter().enumerate() {
                    //values[i_lat][i_lon] = self.value_at(lat_ids[i_lat], lon_ids[i_lon]);
                    values[i_lat][i_lon] = self.interpolate_value_at(
                        *lat,
                        *lon,
                        lat_ids[i_lat],
                        lon_ids[i_lon]
                    );
                }
            }
        }
        //println!("{}, {}", self.lat.len(), self.lon.len());
        values
    }

    fn interpolate_value_at( &self, requested_lat: f64, requested_lon: f64, lat_idx: usize, lon_idx: usize) ->  f32 {

        let mut can_interp_lon = (lon_idx > 0 || self.lon[lon_idx] < requested_lon) 
               && (lon_idx < self.lon.len()-1 || self.lon[lon_idx] > requested_lon);
        let mut can_interp_lat = (lat_idx > 0 || self.lat[lat_idx] < requested_lat) 
               && (lat_idx < self.lat.len()-1 || self.lat[lat_idx] > requested_lat);

        can_interp_lon &= requested_lon != self.lon[lon_idx];
        can_interp_lat &= requested_lat != self.lat[lat_idx];

        let interp_between = | va: f32, a: f64, vb: f32, b: f64, x: f64 | -> f32 {
            if (a - b).abs() > 10. && (x -a).abs() > 10. && (x-b).abs() > 10. {
                ((va as f64 * (x - b).abs() + vb as f64 * (x - a).abs()) / (a - b).abs()) as f32
            } else { 
                if (x-a).abs() < (x-b).abs() {
                    return va;
                }
                return vb;
            }
        };
        let other_lon_idx: usize = if requested_lon > self.lon[lon_idx] {
            lon_idx +1 } else { lon_idx -1 }; 
        let other_lat_idx: usize = if requested_lat > self.lat[lat_idx] {
            lat_idx +1 } else { lat_idx -1 }; 

        if can_interp_lon && can_interp_lat {
            let val_aa = self.value_at(lat_idx, lon_idx);
            let val_ab = self.value_at(lat_idx, other_lon_idx);
            let val_ba = self.value_at(other_lat_idx, lon_idx);
            let val_bb = self.value_at(other_lat_idx, other_lon_idx);
            let a = interp_between(
                val_aa,
                self.lon[lon_idx],
                val_ab,
                self.lon[other_lon_idx],
                requested_lon
            );
            let b = interp_between(
                val_ba,
                self.lon[lon_idx],
                val_bb,
                self.lon[other_lon_idx],
                requested_lon
            );

            return interp_between(
                a,
                self.lat[lat_idx],
                b,
                self.lat[other_lat_idx],
                requested_lat
            );
        } else if can_interp_lon {
            return interp_between(
                self.value_at(lat_idx, lon_idx),
                self.lon[lon_idx],
                self.value_at(lat_idx, other_lon_idx),
                self.lon[other_lon_idx],
                requested_lon
            );
        } else if can_interp_lat {
            return interp_between(
                self.value_at(lat_idx, lon_idx),
                self.lat[lat_idx],
                self.value_at(lat_idx, lon_idx),
                self.lat[other_lat_idx],
                requested_lat
            );
        } else {
            return self.value_at(lat_idx, lon_idx);
        }
    }

    #[inline]
    /// Return the value of self.values as if it was a bi-dimensional array.
    fn value_at(&self, lat_idx: usize, lon_idx: usize) -> f32 {
        return self.values[self.lon.len() * lat_idx + lon_idx];
    }

    /// Creates up to 4 tiles, representing the n+1 zoom level using self.values
    pub fn sub_tiledata(&self) -> Vec<Self> {
        let mut sub_tiledata: Vec<Self> = Vec::new();
        let base_x = self.tile.x * 2;
        let base_y = self.tile.y * 2;
        let z = self.tile.z + 1;
        for x in  base_x..(base_x + 2) {
            for y in base_y..(base_y + 2) {
                let tile = Tile { x: x, y: y, z: z };
                let xy = tile.xy_bounds();

                // get min latitude index
                let mut i_lat_min: usize = 0;
                for (i, lat) in self.lat.iter().enumerate() {
                    if *lat >= xy.south {
                        i_lat_min = if i > 0 { i - 1 } else { 0 };
                        break;
                    }
                }
                // get max latitude index
                let mut i_lat_max: usize = 0;
                for (i, lat) in self.lat.iter().rev().enumerate() {
                    if *lat <= xy.north {
                        i_lat_max = if i != (self.lat.len() - 1 ) { self.lat.len() - i } else { self.lat.len() -1 };
                        break;
                    }
                }
                
                // get min longitude index
                let mut i_lon_min: usize = 0;
                for (i, lon) in self.lon.iter().enumerate() {
                    if *lon >= xy.west {
                        i_lon_min = if i > 0 { i - 1 } else { 0 };
                        break;
                    }
                }
                // get max longitude index
                let mut i_lon_max: usize = 0;
                for (i, lon) in self.lon.iter().rev().enumerate() {
                    if *lon <= xy.east {
                        i_lon_max = if i != (self.lon.len() - 1 ) { self.lon.len() - i } else { self.lon.len() -1 };
                        break;
                    }
                }

                // Extract lat, lon and values using the computed indices
                let subset_lat: Vec<f64> = self.lat[i_lat_min..min(i_lat_max +1, self.lat.len() -1)].to_vec();
                let subset_lon: Vec<f64> = self.lon[i_lon_min..min(i_lon_max +1, self.lon.len() -1)].to_vec();
                let mut subset_values: Vec<f32> = Vec::with_capacity(subset_lat.len() * subset_lon.len());
                for i_lat in i_lat_min..min(i_lat_max +1, self.lat.len() -1) {
                    for i_lon in i_lon_min..min(i_lon_max +1, self.lon.len() -1) {
                        subset_values.push(self.value_at(i_lat, i_lon));
                    }
                }
                
                sub_tiledata.push(
                    Self {
                        lon: subset_lon,
                        lat: subset_lat,
                        values: subset_values,
                        bbox: xy,
                        tile: tile
                    }
                );
            }
        }
        sub_tiledata
    }
}
