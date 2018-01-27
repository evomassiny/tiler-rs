use netcdf;
use netcdf::file::File as NcFile;
use tile::{Tile,LonLatBbox,lat_wgs84_to_meters,lon_wgs84_to_meters};
use tiledata::TileData;
use std::f32;
use utils::search_closest_idx;

/// This Struct provides access to the data within a netCDF file.
pub struct Dataset {
    lat: Vec<f64>,
    lon: Vec<f64>,
    variable_name: String,
    file: NcFile,
}

impl Dataset {
    /// Creates a Dataset instance from a path to a netCDF file, and the name of some required
    /// variable.
    ///
    /// #Args
    ///  * `latitude` name of the latitude variable
    ///  * `longitude` name of the longitude variable
    ///  * `variable` name of the variable to render
    ///  * `file_path` path to the netCDF file.
    ///
    /// # netCDF Fformat expected
    /// The netCDF file must comply to the following rules:
    ///
    /// * The longitude and latitude variable must be sorted in ascending order.
    /// * The longitude and latitude variable must be projected in *WGS 84 (srs 4326)*.
    /// * values of `variable` must be bi-dimensionals (lat, lon)
    ///
    pub fn new(latitude: &str, longitude: &str, variable: &str, file_path: &str) -> Result<Self,String> {
        let file = netcdf::open(file_path)?;
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
        let i_lon_min: usize = search_closest_idx(&self.lon, &bbox.west)
            .ok_or(format!("Longitude error"))?; 
        let i_lon_max: usize = search_closest_idx(&self.lon, &bbox.east)
            .ok_or(format!("Longitude error"))?; 
        // get latitude indices containing the tile data
        let i_lat_max: usize = search_closest_idx(&self.lat, &bbox.north)
            .ok_or(format!("Latitude error"))?; 
        let i_lat_min: usize = search_closest_idx(&self.lat, &bbox.south)
            .ok_or(format!("Latitude error"))?; 

        // Extract data from the netCDF Dataset
        if let Some(variable) = self.file.root.variables.get(&self.variable_name) {
            
            // Compute values slice size (must be > 0)
            let slice_size = [
                i_lat_max - i_lat_min + 1,
                i_lon_max - i_lon_min + 1
            ];

            let mut var_values = variable.values_at(
                &[i_lat_min, i_lon_min],    // start of the data slice
                &slice_size                 // size of the data slice
            )?;
            // Filter fill_values
            let tile_values: Vec<f32> = match self.get_fill_value() {
                Some(fill_value) => { 
                    for v in var_values.iter_mut() {
                        if *v == fill_value { 
                            *v = f32::NAN;
                        }
                    }
                    var_values
                },
                None => { var_values }
            };
            // Convert longitude and latitude into meters
            let lon: Vec<f64> = self.lon[i_lon_min..i_lon_max + 1]
                .iter()
                .map(|x| lon_wgs84_to_meters(*x))
                .collect();
            let lat: Vec<f64> = self.lat[i_lat_min..i_lat_max + 1]
                .iter()
                .map(|x| lat_wgs84_to_meters(*x))
                .collect();

            return Ok(
                TileData {
                    lon: lon,
                    lat: lat,
                    values: tile_values,
                    bbox: tile.xy_bounds(),
                    tile: Tile {x: tile.x, y: tile.y, z: tile.z }
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
