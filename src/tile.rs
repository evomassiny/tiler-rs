use std::f64::{self, consts};

const EARTH_RADIUS: f64 = 6378137.0;
const PERIMETER: f64 = EARTH_RADIUS * 2. *  consts::PI;

#[allow(dead_code)]
/**
 * Turns WGS 84 coordinates into WebMercator
 * tiles numbering
 */
pub fn lon_lat_to_tile(lon: f64, lat: f64, zoom: u32) -> (u32, u32, u32) {
    let lat = lat.to_radians();
    let n = 2_f64.powf(zoom as f64);
    let xtile: f64 = ((lon + 180.0) / 360.0) * n;
    let ytile: f64 = (1.0 - (lat.tan() + (1.0 / lat.cos())).ln() / consts::PI) / 2.0 * n;
    (xtile.floor() as u32, ytile.floor() as u32, zoom)
}

/**
 * Turns WebMercator coordinates into WGS 84 coordinates
 */
pub fn tile_to_wgs84(x: u32, y: u32, z: u32) -> (f64, f64){
    let (x, y) = (x as f64, y as f64);
    let n = 2_f64.powf(z as f64);
    let lon_deg = x / n * 360.0 - 180.0;
    let lat_deg = (consts::PI * (1.0 - 2.0 * y / n)).sinh().atan().to_degrees();
    (lon_deg, lat_deg)
}

/**
 * Turns tiles coordinates into WebMercator (EPSG:3857)
 */
pub fn tile_to_3857(x: u32, y: u32, z: u32) -> (f64, f64){
    let (x, y) = (x as f64, y as f64);
    let n = 2_f64.powf(z as f64);
    let resolution = PERIMETER / n;
    let x_meter = (x * resolution) - PERIMETER / 2.;
    let y_meter = -(y * resolution) + PERIMETER / 2.;
    (x_meter, y_meter)
}

/**
 * Turns WGS84 longitude into meters (Spherical mercator)
 */
pub fn lon_wgs84_to_meters(lon: f64) -> f64 {
    EARTH_RADIUS * lon.to_radians()
}
/**
 * Turns WGS84 latitude into meters (Spherical mercator)
 */
pub fn lat_wgs84_to_meters(lat: f64) -> f64 {
    //EARTH_RADIUS * (consts::PI / 4. + lat.to_radians() / 2.).tan().ln()
    EARTH_RADIUS * lat.to_radians().sin().atanh()
}

/**
 * Turns WGS84 coordinates into meters (Spherical mercator)
 */
pub fn wgs84_to_meters(lon: f64, lat: f64) -> (f64, f64){
    (lon_wgs84_to_meters(lon), lat_wgs84_to_meters(lat))
}


#[derive(Debug, PartialEq)]
pub struct LonLatBbox {
    /// degrees
    pub west: f64,
    /// degrees
    pub south: f64,
    /// degrees
    pub east: f64,
    /// degrees
    pub north: f64,
}
impl LonLatBbox {
    pub fn xy(&self) -> Bbox {
        let (west, north) = wgs84_to_meters(self.west, self.north);
        let (east, south) = wgs84_to_meters(self.east, self.south);
        Bbox {west, south, east, north}
    }
}

#[derive(Debug, PartialEq)]
pub struct Bbox {
    /// meters
    pub west: f64,
    /// meters
    pub south: f64,
    /// meters
    pub east: f64,
    /// meters
    pub north: f64,
}

/// This struct holds basic informations about a Tile.
#[derive(Debug, PartialEq)]
pub struct Tile {
    /// `x` coordinate of a tile
    pub x: u32,
    /// `y` coordinate of a tile
    pub y: u32,
    /// zoom level, (from 0 to 19 included)
    pub z: u32,
}
impl Tile {
    /**
     * Returns the bounding box of self,
     * expressed in WGS 84 
     * */
    pub fn bounds(&self) -> LonLatBbox {
        let (west, north) = tile_to_wgs84(self.x, self.y, self.z);
        let (east, south) = tile_to_wgs84(self.x + 1, self.y + 1, self.z);
        LonLatBbox {west, south, east, north}
    }
    /**
     * Returns Bounding box in Spheriacl mercator coordinates (meters)
     */
    pub fn xy_bounds(&self) -> Bbox {
        //self.bounds().xy()
        let (west, north) = tile_to_3857(self.x, self.y, self.z);
        let (east, south) = tile_to_3857(self.x + 1, self.y + 1, self.z);
        Bbox {west, south, east, north}
    }
}

#[test]
fn test_tile_to_wgs84() {
    assert_eq!((0.0, 66.51326044311186), tile_to_wgs84(2, 1, 2));
    assert_eq!((270.0, -85.0511287798066), tile_to_wgs84(5, 4, 2));
    assert_eq!((-9.140625, 53.33087298301705), tile_to_wgs84(486, 332, 10));
}

#[test]
fn test_lon_lat_to_tile() {
    assert_eq!((16, 14, 5), lon_lat_to_tile(10.0, 20.0, 5));
    assert_eq!((15, 14, 5), lon_lat_to_tile(-10.0, 20.0, 5));
    assert_eq!((23, 7, 5), lon_lat_to_tile(80.0, 70.0, 5));
    assert_eq!((1, 0, 1), lon_lat_to_tile(80.0, 70.0, 1));
}

#[test]
fn test_bounds() {
    let bbox = LonLatBbox {
        west: 78.75, 
        south: 66.51326044311186, 
        east: 90.0, 
        north: 70.61261423801925
    };
    let tile = Tile {
        x: 23,
        y: 7,
        z: 5,
    };
    assert_eq!(bbox, tile.bounds());
}
