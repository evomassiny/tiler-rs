use tile::{Tile,Bbox};

pub const TILE_SIZE: usize = 256;

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
        let lat_inc: f64 = (self.bbox.north - self.bbox.south) / (TILE_SIZE as f64);

        let mut current_idx: usize = 0;
        let mut current_lat: f64 = self.bbox.south;
        for i in 0..TILE_SIZE {
            // If the closest lat value is the biggest one, 
            // it won't change over the next iteration
            if current_idx == self.lat.len() -1 {
                for following_i in i..TILE_SIZE {
                    lat_ids[following_i] = current_idx;
                }
                break;
            }
            while (self.lat[current_idx] - current_lat).abs() >  (self.lat[current_idx + 1] - current_lat).abs() {
                current_idx += 1;
                if current_idx == self.lat.len() -1 {
                    break;
                }
            }
            lat_ids[i] = current_idx;
            current_lat += lat_inc;
        }

        // fetch nearest longitude indices
        let mut lon_ids: [usize; TILE_SIZE] = [0; TILE_SIZE];
        let lon_inc: f64 = (self.bbox.east - self.bbox.west) / (TILE_SIZE as f64);

        let mut current_idx: usize = 0;
        let mut current_lon: f64 = self.bbox.west;
        for i in 0..TILE_SIZE {
            if current_idx == self.lon.len() -1 {
                for following_i in i..TILE_SIZE {
                    lon_ids[following_i] = current_idx;
                }
                break;
            }
            while (self.lon[current_idx] - current_lon).abs() >  (self.lon[current_idx + 1] - current_lon).abs() {
                current_idx += 1;
                if current_idx == self.lon.len() -1 {
                    break;
                }
            }
            lon_ids[i] = current_idx;
            current_lon += lon_inc;
        }

        // pick values using precomputed indices
        let mut values = [[0_f32; TILE_SIZE]; TILE_SIZE];
        for i_lat in 0..TILE_SIZE {
            for i_lon in 0..TILE_SIZE {
                values[i_lat][i_lon] = self.value_at(lat_ids[i_lat], lon_ids[i_lon]);
            }
        }
        values
    }

    /// Return the value of self.values as if it was a bi-dimensional array.
    fn value_at(&self, lat_idx: usize, lon_idx: usize) -> f32 {
        return self.values[self.lon.len() * lat_idx + lon_idx];
    }

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

                let subset_lat: Vec<f64> = self.lat[i_lat_min..(i_lat_max +1)].to_vec();
                let subset_lon: Vec<f64> = self.lon[i_lon_min..(i_lon_max +1)].to_vec();
                let mut subset_values: Vec<f32> = Vec::with_capacity(subset_lat.len() * subset_lon.len());
                let mut lat_idx: usize = 0;
                for i_lat in i_lat_min..(i_lat_max + 1) {
                    let mut lon_idx: usize = 0;
                    for i_lon in i_lon_min..(i_lon_max + 1) {
                        subset_values[lat_idx * subset_lat.len() + lon_idx] = self.value_at(i_lat, i_lon);
                        lon_idx += 1;
                    }
                    lat_idx += 1;
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

#[test]
fn nearest_interp() {
    let tile_data = TileData {
        lat: vec![1., 2., 3.],
        lon: vec![0., 1., 2.],
        values: vec![0., 0., 0., 0., 2., 0., 0., 0., 0.],
        bbox: Bbox {
            west: 0.5001,
            south: 1.5001,
            east: 1.4999,
            north: 2.4999,
        }, 
        tile: Tile { x: 1, y:1, z: 1 }    // dummies
    };
    let grid = tile_data.to_tile_grid();
    for i_lat in 0..TILE_SIZE {
        for i_lon in 0..TILE_SIZE {
            assert_eq!(
                grid[i_lat][i_lon], 2.,
                "test ilat:{}, ilon:{}", i_lat, i_lon
            );

        }
    }
}
