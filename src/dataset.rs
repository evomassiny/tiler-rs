use netcdf;
use netcdf::attribute::AttrValue;
use netcdf::file::File as NcFile;
use tile::{lat_wgs84_to_meters, lon_wgs84_to_meters, wgs84_to_meters, Bbox, Tile};
//use tile::{Tile,LonLatBbox,lat_to_pixel,lon_to_pixel};
use std::f32;
use tiledata::TileData;
use utils::{search_closest_idx, search_closest_idx_below, search_closest_idx_over};

fn format_error(error: netcdf::error::Error) -> String {
    format!("{:?}", error)
}

/// This Struct provides access to the data within a netCDF file.
pub struct Dataset {
    // meter (Web Mercator)
    lat: Vec<f64>,
    min_lat: f64,
    max_lat: f64,
    // meter (Web Mercator)
    lon: Vec<f64>,
    min_lon: f64,
    max_lon: f64,
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
    pub fn new(
        latitude: &str,
        longitude: &str,
        variable: &str,
        file_path: &str,
    ) -> Result<Self, String> {
        let file = netcdf::open(file_path).map_err(format_error)?;
        let root = file.root().ok_or("No root group")?;

        let lat_var = root.variable(latitude).ok_or("No latitude")?;

        let size = lat_var.len();
        let mut lat: Vec<f64> = unsafe {
            let mut v = Vec::with_capacity(size);
            v.set_len(size);
            v
        };
        lat_var
            .values_to(lat.as_mut_slice(), None, None)
            .map_err(format_error)?;
        // convert WGS84 to WebMercator
        for y in lat.iter_mut() {
            *y = lat_wgs84_to_meters(*y);
        }

        let lon_var = root.variable(longitude).ok_or("No lonitude")?;
        let size = lon_var.len();
        let mut lon: Vec<f64> = unsafe {
            let mut v = Vec::with_capacity(size);
            v.set_len(size);
            v
        };
        lon_var
            .values_to(lon.as_mut_slice(), None, None)
            .map_err(format_error)?;
        // convert WGS84 to WebMercator
        for x in lon.iter_mut() {
            *x = lon_wgs84_to_meters(*x);
        }
        Ok(Self {
            min_lat: lat[0].min(lat[lat.len() - 1]),
            max_lat: lat[0].max(lat[lat.len() - 1]),
            lat: lat,
            min_lon: lon[0].min(lon[lon.len() - 1]),
            max_lon: lon[0].max(lon[lon.len() - 1]),
            lon: lon,
            variable_name: variable.into(),
            file: file,
        })
    }

    /**
     * Get the fill value of the dataset
     */
    pub fn get_fill_value(&self) -> Option<f32> {
        if let Some(var) = self.file.root()?.variable(&self.variable_name) {
            if let Some(attr) = var.attribute("_FillValue") {
                match attr.value() {
                    Ok(AttrValue::Float(x)) => Some(x),
                    Ok(AttrValue::Double(x)) => Some(x as f32),
                    _ => None,
                };
            }
        }
        None
    }

    /**
     * Check if the bounding box is not strictly outside
     * the lon/lat range of the dataset
     */
    fn contains_bbox(&self, bbox: &Bbox) -> bool {
        if bbox.west <= self.min_lon && bbox.east <= self.min_lon {
            return false;
        }
        if bbox.west >= self.max_lon && bbox.east >= self.max_lon {
            return false;
        }
        if bbox.south <= self.min_lat && bbox.north <= self.min_lat {
            return false;
        }
        if bbox.south >= self.max_lat && bbox.north >= self.max_lat {
            return false;
        }
        return true;
    }

    /**
     * Check if the point (lat, lon in WebMercator EPSG:3857)  is contained in the dataset extend.
     */
    fn contains_point(&self, lat: f64, lon: f64) -> bool {
        if lon < self.min_lon || lon > self.max_lon {
            return false;
        }
        if lat < self.min_lat || lat > self.max_lat {
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
        let mut i_lon_min: usize =
            search_closest_idx_below(&self.lon, bbox.west).ok_or(format!("Longitude error"))?;
        let mut i_lon_max: usize =
            search_closest_idx_over(&self.lon, bbox.east).ok_or(format!("Longitude error"))?;
        if i_lon_max < i_lon_min {
            let tmp = i_lon_max;
            i_lon_max = i_lon_min;
            i_lon_min = tmp;
        }

        // get latitude indices containing the tile data
        let mut i_lat_min: usize =
            search_closest_idx_below(&self.lat, bbox.south).ok_or(format!("Latitude error"))?;
        let mut i_lat_max: usize =
            search_closest_idx_over(&self.lat, bbox.north).ok_or(format!("Latitude error"))?;
        if i_lat_max < i_lat_min {
            let tmp = i_lat_max;
            i_lat_max = i_lat_min;
            i_lat_min = tmp;
        }
        // Extract data from the netCDF Dataset
        if let Some(variable) = self
            .file
            .root()
            .ok_or("No root group !")?
            .variable(&self.variable_name)
        {
            // Compute values slice size (must be > 0)
            let slice_size = [i_lat_max - i_lat_min + 1, i_lon_max - i_lon_min + 1];
            let size = slice_size[0] * slice_size[1];

            let mut var_values: Vec<f32> = unsafe {
                let mut v = Vec::with_capacity(size);
                v.set_len(size);
                v
            };
            variable
                .values_to(
                    var_values.as_mut_slice(),
                    Some(&[i_lat_min, i_lon_min]), // start of the data slice
                    Some(&slice_size),             // size of the data slice
                )
                .map_err(format_error)?;
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

            return Ok(TileData {
                min_lon: lon[0].min(lon[lon.len() - 1]),
                max_lon: lon[0].max(lon[lon.len() - 1]),
                lon: lon,
                min_lat: lat[0].min(lat[lat.len() - 1]),
                max_lat: lat[0].max(lat[lat.len() - 1]),
                lat: lat,
                values: var_values,
                bbox: bbox,
                tile: Tile {
                    x: tile.x,
                    y: tile.y,
                    z: tile.z,
                },
            });
        }
        Err("Error while fetching tile, no variable found".into())
    }

    /// Return the value stored at (lat, lon)
    pub fn value_at_coordinates(&self, lat: f64, lon: f64) -> Result<f32, String> {
        // transform (lat, lon) into Web Mercator (as self.lat and self.lon)
        let (x, y) = wgs84_to_meters(lon, lat);
        if self.contains_point(y, x) {
            // fetch the closest point in the dataset
            let lon_idx: usize =
                search_closest_idx(&self.lon, x).ok_or_else(|| format!("longitude error"))?;
            let lat_idx: usize =
                search_closest_idx(&self.lat, y).ok_or_else(|| format!("latitude error"))?;
            // extract it value
            if let Some(variable) = self
                .file
                .root()
                .ok_or("No root !")?
                .variable(&self.variable_name)
            {
                let value = variable
                    .value::<f32>(Some(&[lat_idx, lon_idx]))
                    .map_err(format_error)?;
                if let Some(fill_value) = self.get_fill_value() {
                    if value == fill_value {
                        return Ok(f32::NAN);
                    }
                }
                return Ok(value);
            }
        }
        Err("Dataset error".into())
    }
}

#[test]
fn dataset_creation() {
    let dataset_path = "./examples_data/wind_magnitude_reduced.nc";
    let dataset = Dataset::new("latitude", "longitude", "wind_magnitude", dataset_path);
    assert!(dataset.is_ok());
}

#[test]
fn test_data_fetch() {
    let dataset_path = "./examples_data/wind_magnitude_reduced.nc";
    let dataset = Dataset::new("latitude", "longitude", "wind_magnitude", dataset_path).unwrap();
    let tile = Tile { x: 15, y: 15, z: 9 };
    let values = dataset.get_tile_data(&tile);
    assert!(&values.is_ok());
}
