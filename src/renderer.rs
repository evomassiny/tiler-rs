use std::path::Path;
use dataset::Dataset;
use tiledata::{TILE_SIZE};
use tile::Tile;
use tiledata::TileData;
use colormap::{ColorMap,rgb};
use scale::{Scale,normalize};
use image;

/// This struct represents an image tile,
/// it holds all the pixel values needed to build 
/// an image file (PNG) from it.
pub struct ImgTile {
    /// Array of pixel values (flattened) 
    pub pixels: [u8; 4 * TILE_SIZE * TILE_SIZE],
    /// Web mercator x coordinate of the tile
    pub x: u32,
    /// Web mercator y coordinate of the tile
    pub y: u32,
    /// Zoom level
    pub z: u32,
}
impl ImgTile {
    /// Export the ImgTile as a PNG file.
    pub fn save(&self, path: &str) {
        let _ = image::save_buffer(
            &Path::new(path),
            &self.pixels,
            TILE_SIZE as u32,
            TILE_SIZE as u32,
            image::RGBA(8)
        );
    }
}

/// Provides convenient functions to render a `Dataset` instance into `ImgTile`s
pub struct Renderer {
    color_map: ColorMap,
    scale: Scale,
    dataset: Dataset
}
impl Renderer {
    /** Create a `Renderer` instance from a dataset.
     *
     * # Args
     * * `dataset`: A dataset instance (which wraps an netCDF file)
     * * `min`: the minimum value of the colorbar
     * * `max`: the maximum value of the colorbar
     * * `color_map`: a ColorMap variant, which defines the *value* => *color* mapping
     */
    pub fn from_dataset(dataset: Dataset, scale: Scale, color_map: ColorMap)
            -> Result<Self, String> {
        Ok(
            Self {
                color_map: color_map,
                scale: scale,
                dataset: dataset
            }
        )
    }

    /**
     * Returns a pixel value (RGBA) from a value, according to the 
     * renderer colormap, and the scale
     */
    #[inline]
    pub fn value_to_rgba(&self, value: f32) -> [u8; 4] {
        if value.is_nan() {
            return [0u8, 0u8, 0u8, 0u8];
        }
        let scaled_value = normalize(&self.scale, value);
        let rgb = rgb(scaled_value, &self.color_map);
        [rgb[0], rgb[1], rgb[2], 255u8]
    }

    fn values_to_colors(&self, values: [[f32; TILE_SIZE]; TILE_SIZE])
            -> [u8; 4 * TILE_SIZE * TILE_SIZE] {
        let mut colors = [0u8; 4* TILE_SIZE * TILE_SIZE];
        let mut count: usize = 0;
        // iter latitude in reverse, to fit the image X,Y orientation
        for i_lat in (0..TILE_SIZE).rev() {
            for i_lon in 0..TILE_SIZE {
                let rgba = self.value_to_rgba(values[i_lat][i_lon]);
                colors[count + 0] = rgba[0];
                colors[count + 1] = rgba[1];
                colors[count + 2] = rgba[2];
                colors[count + 3] = rgba[3];
                count += 4;
            }
        }
        colors
    }

    /// Return the value stored at (lat, lon)
    pub fn value_at_coordinates(&self, lat: f64, lon: f64) -> Result<f32,String> {
        self.dataset.value_at_coordinates(lat, lon)
    }

    /**
     * Render a Tile into an ImgTile.
     * 
     * #Details
     *
     * Extract values from the renderer dataset, 
     * interpolate them into a TILE_SIZE * TILE_SIZE grid,
     * and convert them into pixel values.
     */
    pub fn render_tile(&self, tile: &Tile) -> Result<ImgTile, String> {
        let tile_data = self.dataset.get_tile_data(tile)?;
        let data = tile_data.to_tile_grid();
        let colors = self.values_to_colors(data);
        Ok(
            ImgTile {
                pixels: colors,
                x: tile.x,
                y: tile.y,
                z: tile.z,
            }
        )
    }

    /// This function render a Tildata and its `level` sub-levels into ImgTile,
    /// by *RECURSIVELY* calling itself using `data.sub_tiledata`.
    fn render_n_tiledata_zoom(&self, data: &TileData, level: u8) -> Vec<ImgTile> {
        let mut imgs: Vec<ImgTile> = Vec::new();
        imgs.push(
            ImgTile {
                pixels: self.values_to_colors(data.to_tile_grid()),
                x: data.tile.x,
                y: data.tile.y,
                z: data.tile.z,
            }
        );

        if level > 0 {
            for sub_data in data.sub_tiledata() {
                imgs.extend(self.render_n_tiledata_zoom(&sub_data, level -1));
            }
        }
        return imgs;

    }

    /// This function renders a tile and its `level` sub-levels into ImgTile.
    /// It only extracts values from the dataset once, and recursively renders `level` levels 
    /// of tiles using those values.
    pub fn render_n_level_tile(&self, tile: &Tile, level: u8) -> Result<Vec<ImgTile>, String> {
        let tile_data = self.dataset.get_tile_data(tile)?;
        return Ok(self.render_n_tiledata_zoom(&tile_data, level));
    }


}
