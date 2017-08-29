use netcdf;
use netcdf::file::File as NcFile;
use tile::{Tile,LonLatBbox,lat_wgs84_to_meters,lon_wgs84_to_meters};
use tiledata::TileData;

pub struct Dataset {
    lat: Vec<f64>,
    lon: Vec<f64>,
    variable_name: String,
    file: NcFile,
}

impl Dataset {
    pub fn new(latitude: &str, longitude: &str, variable: &str, file_name: &str) -> Result<Self,String> {
        let file = netcdf::open(file_name)?;
        let lat: Vec<f64> = file.root.variables
            .get(latitude)
            .ok_or("No latitude")?
            .values()?;
        let lon: Vec<f64> = file.root.variables
            .get(longitude)
            .ok_or("No longitude")?
            .values()?;
        Ok(
            Self {
                lat: lat,
                lon: lon,
                variable_name: variable.into(),
                file: file,
            }
        )
    }

    /**
     * Get the fill value of the dataset
     */
    pub fn get_fill_value(&self) -> Option<f32> {
        if let Some(var) =  self.file.root.variables.get(&self.variable_name) {
            if let Some(attr) = var.attributes.get("_FillValue") {
                 return attr.get_float(true).ok();
            }
        }
        None
    }

    /**
     * Check if the bounding box is not strictly outside
     * the lon/lat range of the dataset
     */
    fn contains_bbox(&self, bbox: &LonLatBbox) -> bool {
        let (lon_min, lon_max) = (self.lon[0], self.lon[self.lon.len() -1 ]);
        let (lat_min, lat_max) = (self.lat[0], self.lat[self.lat.len() -1 ]);
        if bbox.west <= lon_min && bbox.east <= lon_min {
            return false;
        }
        if bbox.west >= lon_max && bbox.east >= lon_max {
            return false;
        }
        if bbox.south <= lat_min && bbox.north <= lat_min {
            return false;
        }
        if bbox.south >= lat_max && bbox.north >= lat_max  {
            return false;
        }
        return true;
    }

    /**
     * Extract data from the netCDF dataset
     * and pack it into a TileData
     */
    pub fn get_tile_data(&self, tile: &Tile) -> Result<TileData, String> {
        let bbox = tile.bounds();
        if !self.contains_bbox(&bbox) {
            return Err("tile outside range".into());
        }

        // get longitude indices containing the tile data
        let (mut i_lon_min, mut i_lon_max) = (0, self.lon.len() - 1);
        for i in 0..self.lon.len() {
            if bbox.west == self.lon[i] {
                i_lon_min = i;
                break;
            } else if bbox.west > self.lon[i] {
                i_lon_min = if i != 0 { i - 1 } else { 0 };
                break;
            }
        }
        for i in i_lon_min..self.lon.len() {
            if bbox.east <= self.lon[i] {
                i_lon_max = i;
                break;
            }

        }
        // get latitude indices containing the tile data
        let (mut i_lat_min, mut i_lat_max) = (0, self.lat.len() - 1);
        for i in 0..self.lat.len() {
            if bbox.south == self.lat[i] {
                i_lat_min = i;
                break;
            } else if bbox.south > self.lat[i] {
                i_lat_min = if i != 0 { i - 1 } else { 0 };
                break;
            }
        }
        for i in i_lat_min..self.lat.len() {
            if bbox.north <= self.lat[i] {
                i_lat_max = i;
                break;
            }

        }
        // Extract data from the netCDF Dataset
        if let Some(variable) = self.file.root.variables.get(&self.variable_name) {
            
            // Compute values slice size (must be > 0)
            let slice_size = [
                i_lat_max - i_lat_min + 1,
                i_lon_max - i_lon_min + 1
            ];

            let tile_values = variable.values_at(&[i_lat_min, i_lon_min], &slice_size)?;
            // Convert longitude and latitude into meters
            let lon: Vec<f64> = self.lon[i_lon_min..i_lon_max + 1]
                .iter().map(|x| lon_wgs84_to_meters(*x)).collect();
            let lat: Vec<f64> = self.lat[i_lat_min..i_lat_max + 1]
                .iter().map(|x| lat_wgs84_to_meters(*x)).collect();

            return Ok(
                TileData {
                    lon: lon,
                    lat: lat,
                    values: tile_values,
                    bbox: tile.xy_bounds()
                }
            );
        }
        Err("Error while fetching tile, no variable found".into())
    }
}

#[test]
fn dataset_creation() {
    let dataset_path = "./examples_data/wind_magnitude_reduced.nc";
    let dataset = Dataset::new(
        "latitude", 
        "longitude",
        "wind_magnitude",
        dataset_path
    );
    assert!(dataset.is_ok());
}

#[test]
fn test_data_fetch() {
    let dataset_path = "./examples_data/wind_magnitude_reduced.nc";
    let dataset = Dataset::new(
        "latitude", 
        "longitude",
        "wind_magnitude",
        dataset_path
    ).unwrap();
    let tile = Tile {
        x: 15,
        y: 15,
        z: 9,
    };
    let values = dataset.get_tile_data(&tile);
    assert!(&values.is_ok());
}
