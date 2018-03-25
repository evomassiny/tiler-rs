use netcdf;
use netcdf::file::File as NcFile;
use tile::{Tile,Bbox,wgs84_to_meters,lat_wgs84_to_meters,lon_wgs84_to_meters};
//use tile::{Tile,LonLatBbox,lat_to_pixel,lon_to_pixel};
use tiledata::TileData;
use std::f32;
use utils::search_closest_idx;

/// This Struct provides access to the data within a netCDF file.
pub struct Dataset {
    // meter (Web Mercator)
    lat: Vec<f64>,
    // meter (Web Mercator)
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
        let mut lat: Vec<f64> = file.root.variables
            .get(latitude)
            .ok_or("No latitude")?
            .values()?;
        // convert WGS84 to WebMercator
        for y in lat.iter_mut() {
            *y = lat_wgs84_to_meters(*y);
        }
        let mut lon: Vec<f64> = file.root.variables
            .get(longitude)
            .ok_or("No longitude")?
            .values()?;
        // convert WGS84 to WebMercator
        for x in lon.iter_mut() {
            *x = lon_wgs84_to_meters(*x);
        }
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
    fn contains_bbox(&self, bbox: &Bbox) -> bool {
        let (lon_min, lon_max) = (self.lon[0], self.lon[self.lon.len() -1]);
        let (lat_min, lat_max) = (self.lat[0], self.lat[self.lat.len() -1]);
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
        let bbox = tile.xy_bounds();
        if !self.contains_bbox(&bbox) {
            return Err("tile outside range".into());
        }

        // get longitude indices containing the tile data
        let mut i_lon_min: usize = search_closest_idx(&self.lon, &bbox.west)
            .ok_or(format!("Longitude error"))?; 
        if i_lon_min > 0 && bbox.west <= self.lon[i_lon_min] {
            i_lon_min -= 1;
        }
        let mut i_lon_max: usize = search_closest_idx(&self.lon, &bbox.east)
            .ok_or(format!("Longitude error"))?; 
        if i_lon_max != (self.lon.len() -1) && bbox.east >= self.lon[i_lon_max] {
            i_lon_max += 1;
        }
        // get latitude indices containing the tile data
        let mut i_lat_min: usize = search_closest_idx(&self.lat, &bbox.south)
            .ok_or(format!("Latitude error"))?; 
        if i_lat_min > 0 && bbox.south <= self.lat[i_lat_min] {
            i_lat_min -= 1;
        }
        let mut i_lat_max: usize = search_closest_idx(&self.lat, &bbox.north)
            .ok_or(format!("Latitude error"))?; 
        if i_lat_max != (self.lat.len() -1) && bbox.north >= self.lat[i_lat_max] {
            i_lat_max += 1;
        }

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
            if let Some(fill_value) = self.get_fill_value() {
                for v in var_values.iter_mut() {
                    if *v == fill_value { 
                        *v = f32::NAN;
                    }
                }
            }
            // Pick the associated lon /lat values
            let lon: Vec<f64> = self.lon[i_lon_min..(i_lon_max + 1)]
                .iter()
                .map(|x| *x)
                .collect();
            let lat: Vec<f64> = self.lat[i_lat_min..(i_lat_max + 1)]
                .iter()
                .map(|x| *x)
                .collect();

            return Ok(
                TileData {
                    lon: lon,
                    lat: lat,
                    values: var_values,
                    bbox: bbox,
                    tile: Tile {x: tile.x, y: tile.y, z: tile.z }
                }
            );
        }
        Err("Error while fetching tile, no variable found".into())
    }

    /// Return the value stored at (lat, lon)
    pub fn value_at_coordinates(&self, lat: f64, lon: f64) -> Result<f32, String> {
        // transform (lat, lon) into Web Mercator (as self.lat and self.lon)
        let (x, y) = wgs84_to_meters(lon, lat);
        // fetch the closest point in the dataset
        let lon_idx: usize = search_closest_idx(&self.lon, &x)
            .ok_or_else(|| format!("longitude error"))?;
        let lat_idx: usize = search_closest_idx(&self.lat, &y)
            .ok_or_else(|| format!("latitude error"))?;
        // extract it value
        if let Some(variable) = self.file.root.variables.get(&self.variable_name) {
            let value: f32 = variable.value_at(&[lat_idx, lon_idx])?;
            println!("{}, {}", &lat_idx, &lon_idx);
            if let Some(fill_value) = self.get_fill_value() {
                if value == fill_value {
                    return Ok(f32::NAN);
                }
            }
            return Ok(value);
        }
        Err("Dataset error".into())
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
