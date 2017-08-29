use std::path::Path;
use dataset::Dataset;
use tiledata::{TILE_SIZE};
use tile::Tile;
use tiledata::TileData;
use colormap::{ColorMap,rgb};
use image;

pub struct ImgTile {
    pub pixels: [u8; 4 * TILE_SIZE * TILE_SIZE],
    pub x: u16,
    pub y: u16,
    pub z: u16,
}
impl ImgTile {
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

pub struct Renderer {
    fill_value: f32,
    max_value: f32,
    min_value: f32,
    color_map: ColorMap,
    dataset: Dataset
}
impl Renderer {
    pub fn from_dataset(dataset: Dataset, min: f32, max: f32, color_map: ColorMap) -> Result<Self, String> {
        let fill_value = dataset.get_fill_value().unwrap_or(-1_f32);
        // TODO: read values from the dataset
        Ok(
            Self {
                fill_value: fill_value,
                min_value: min,
                max_value: max,
                color_map: color_map,
                dataset: dataset
            }
        )
    }

    fn to_scale(&self, value: f32) -> f32 {
        if value <= self.min_value {
            return 0.;
        } else if value >= self.max_value {
            return 1.;
        } else {
            return (value -self. min_value) / (self.max_value - self.min_value);
        }
    }

    fn values_to_colors(&self, values: [[f32; TILE_SIZE]; TILE_SIZE]) -> [u8; 4 * TILE_SIZE * TILE_SIZE] {
        let mut colors = [0u8; 4* TILE_SIZE * TILE_SIZE];
        let mut count: usize = 0;
        // iter latitude in reverse, to fit the image X,Y orientation
        for i_lat in (0..TILE_SIZE).rev() {
            for i_lon in 0..TILE_SIZE {
                if values[i_lat][i_lon] == self.fill_value {
                    // mask fill_values
                    colors[count] = 0;
                    colors[count + 1] = 0;
                    colors[count + 2] = 0;
                    colors[count + 3] = 0;
                } else {
                    let value = self.to_scale(values[i_lat][i_lon]);
                    let rgb = rgb(value, &self.color_map);
                    colors[count] = rgb[0];
                    colors[count + 1] = rgb[1];
                    colors[count + 2] = rgb[2];
                    colors[count + 3] = 255;
                }
                count += 4;
            }
        }
        colors
    }

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

    pub fn render_n_level_tile(&self, tile: &Tile, level: u8) -> Result<Vec<ImgTile>, String> {
        let tile_data = self.dataset.get_tile_data(tile)?;
        return Ok(self.render_n_tiledata_zoom(&tile_data, level));
    }


}
