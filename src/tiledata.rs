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

        // Build latitude needed for each pixel
        let lat_inc: f64 = (self.bbox.north -self.bbox.south) / (TILE_SIZE as f64);
        let lats: Vec<f64> = (0..TILE_SIZE).map(|i| {
            self.bbox.south + lat_inc * (0.5 + i as f64)
        }).collect();

        // Build longitude needed for each pixel
        let lon_inc: f64 = (self.bbox.east - self.bbox.west) / (TILE_SIZE as f64);
        let lons: Vec<f64> = (0..TILE_SIZE).map(|i| {
            self.bbox.west + lon_inc * (0.5 + i as f64)
        }).collect();

        // Find out values boundaries
        let lat_min: f64 = self.lat[0] - 0.5 * lat_inc;
        let lat_max: f64 = self.lat[self.lat.len() - 1] + 0.5 * lat_inc;
        let lon_min: f64 = self.lon[0] - 0.5 * lon_inc;
        let lon_max: f64 = self.lon[self.lon.len() - 1] + 0.5 * lon_inc;
        // this closure returns true if the lon/lat coordinates 
        // are included in the dataset
        let in_value_extend = | lat: f64, lon: f64 | -> bool {
            lat >= lat_min && lat <= lat_max && lon >= lon_min && lon <= lon_max
        };


        // Build output values
        let mut values: [[f32; TILE_SIZE]; TILE_SIZE] = [
            [f32::NAN; TILE_SIZE];
            TILE_SIZE
        ];
        // directly average the nearest data or interpole it
        // depending of the number of data available
        if self.values.len() > TILE_SIZE * TILE_SIZE {
            // average the data contained in the pixel extend
            for (i_lat, lat) in lats.iter().enumerate() {
                for (i_lon, lon) in lons.iter().enumerate() {
                    if in_value_extend(*lat, *lon) {
                        values[i_lat][i_lon] = self.resample_average(*lat, *lon, lat_inc, lon_inc);
                    }
                }
            }
        } else {
            // interpolate each pixel value
            for (i_lat, lat) in lats.iter().enumerate() {
                for (i_lon, lon) in lons.iter().enumerate() {
                    if in_value_extend(*lat, *lon) {
                        values[i_lat][i_lon] = self.interpolate_value_at(*lat, *lon);
                    }
                }
            }
        }
        values
    }

    /// Fetch and compute the average of all value represented by a single pixel
    fn resample_average(&self, requested_lat: f64, requested_lon: f64, lat_inc: f64, lon_inc: f64) ->  f32 {

        // get the index ot the lowest bound
        let min_lat_idx = search_closest_idx(
            &self.lat,
            &(requested_lat - lat_inc / 2.)
        ).unwrap();
        let min_lon_idx = search_closest_idx(
            &self.lon,
            &(requested_lon - lon_inc / 2.)
        ).unwrap();

        // get the index ot the highest bound
        let mut max_lat_idx = min_lat_idx;
        while max_lat_idx != (self.lat.len() -1) 
            && self.lat[max_lat_idx] < (requested_lat + lat_inc / 2.) {
            max_lat_idx += 1;
        }
        let mut max_lon_idx = min_lon_idx;
        while max_lon_idx != (self.lon.len() -1) 
            && self.lon[max_lon_idx] < (requested_lon + lon_inc / 2.) {
            max_lon_idx += 1;
        }
        // FETCH values inside the square defined by the bounds
        let mut values: Vec<f32> = Vec::with_capacity(
            (max_lon_idx - min_lon_idx) * (max_lat_idx - min_lat_idx)
        );
        for lat_idx in min_lat_idx..(max_lat_idx+1) {
            for lon_idx in min_lon_idx..(max_lon_idx+1) {
                values.push(self.value_at(lat_idx, lon_idx));
            }
        }
        // compute the average of it
        let mut pixel_value: f32 = 0.;
        let mut valid_count: f32 = 0.;
        for value in values {
            // ignore NAN
            if !value.is_nan() {
                pixel_value += value;
                valid_count += 1.;
            }
        }
        if valid_count == 0. {
            return f32::NAN;
        }
        (pixel_value / valid_count)
    }

    /// This function fetch and interpolate the data from self.value, self.lon, self.lat
    /// at the requested lat / lon.
    /// It basically performs a bilinear interpolation
    fn interpolate_value_at(&self, requested_lat: f64, requested_lon: f64) ->  f32 {
        // fetch nearest longitude / latitude indices
        let lat_idx = search_closest_idx(&self.lat, &requested_lat).unwrap();
        let lon_idx = search_closest_idx(&self.lon, &requested_lon).unwrap();

        // see if we can interpolate the data, we need 4 points inside the bounding
        // box of self.lon and self.lat
        let can_interp_lon = (lon_idx > 0 || self.lon[lon_idx] < requested_lon) 
               && (lon_idx < self.lon.len()-1 || self.lon[lon_idx] > requested_lon);
        let can_interp_lat = (lat_idx > 0 || self.lat[lat_idx] < requested_lat) 
               && (lat_idx < self.lat.len()-1 || self.lat[lat_idx] > requested_lat);

        // this function interpolate a value between 2 other
        // * `a` and `b` are the position of the 2 input points, 
        // * `va` and `vb` are their values.
        // * `x` is the position where the output value will be expressed
        let interp_between = | va: f32, a: f64, vb: f32, b: f64, x: f64 | -> f32 {
            // If the value at the point `a` is NaN, 
            // and x` is closer to `a` than `b` don't interpolate.
            if va.is_nan() && (x - b).abs() < (x - a).abs(){ return vb; }
            // Same for `b`
            if vb.is_nan() && (x - a).abs() < (x - b).abs(){ return va; }
            // perform the interpolation
            let vx =   va * ((x - b).abs() / (a - b).abs()) as f32 
                     + vb * ((x - a).abs() / (a - b).abs()) as f32;
            return vx;
        };

        // get the indices of the other points needed to interpolate
        let other_lon_idx: usize = if requested_lon > self.lon[lon_idx] {
            lon_idx +1 } else { lon_idx -1 }; 
        let other_lat_idx: usize = if requested_lat > self.lat[lat_idx] {
            lat_idx +1 } else { lat_idx -1 }; 

        if can_interp_lon && can_interp_lat {
            // First interpolate linearly the 4 points at the requested longitude
            let a = interp_between(
                self.value_at(lat_idx, lon_idx),
                self.lon[lon_idx],
                self.value_at(lat_idx, other_lon_idx),
                self.lon[other_lon_idx],
                requested_lon
            );
            let b = interp_between(
                self.value_at(other_lat_idx, lon_idx),
                self.lon[lon_idx],
                self.value_at(other_lat_idx, other_lon_idx),
                self.lon[other_lon_idx],
                requested_lon
            );
            // finally interpolate at the requested latitude
            return interp_between(
                a,
                self.lat[lat_idx],
                b,
                self.lat[other_lat_idx],
                requested_lat
            );
        } else if can_interp_lon {
            // only interpolate at the requested longitude
            return interp_between(
                self.value_at(lat_idx, lon_idx),
                self.lon[lon_idx],
                self.value_at(lat_idx, other_lon_idx),
                self.lon[other_lon_idx],
                requested_lon
            );
        } else if can_interp_lat {
            // only interpolate at the requested latitude
            return interp_between(
                self.value_at(lat_idx, lon_idx),
                self.lat[lat_idx],
                self.value_at(lat_idx, lon_idx),
                self.lat[other_lat_idx],
                requested_lat
            );
        } else {
            // If we can't interpolate, returns the nearest value
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
        // TODO: Use binary search
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
