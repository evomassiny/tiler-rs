use std::path::Path;
use dataset::Dataset;
use tiledata::{TILE_SIZE};
use tile::Tile;
use image;

pub struct ImgTile {
    pub pixels: [u8; 2 * TILE_SIZE * TILE_SIZE],
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
            image::GrayA(8)
        );
    }
}

pub struct Renderer {
    fill_value: f32,
    max_value: f32,
    min_value: f32,
    dataset: Dataset
}
impl Renderer {
    pub fn from_dataset(dataset: Dataset, min: f32, max: f32) -> Result<Self, String> {
        let fill_value = dataset.get_fill_value().unwrap_or(-1_f32);
        // TODO: read values from the dataset
        Ok(
            Self {
                fill_value: fill_value,
                min_value: min,
                max_value: max,
                dataset: dataset
            }
        )
    }

    fn values_to_colors(&self, values: [[f32; TILE_SIZE]; TILE_SIZE]) -> [u8; 2 * TILE_SIZE * TILE_SIZE] {
        let mut colors = [0u8; 2* TILE_SIZE * TILE_SIZE];
        let scale = |x| ((x - self.min_value) / (self.max_value - self.min_value));
        let mut count: usize = 0;
        // iter latitude in reverse, to fit the image X,Y orientation
        for i_lat in (0..TILE_SIZE).rev() {
            for i_lon in 0..TILE_SIZE {
                if values[i_lat][i_lon] == self.fill_value {
                    // mask fill_values
                    colors[count] = 0;
                    colors[count + 1] = 0;
                } else {
                    colors[count] = ((scale(values[i_lat][i_lon]) * 255.) as u8) % 255;
                    colors[count + 1] = 255;
                    //println!("{}: {}", values[i_lat][i_lon], colors[count]);
                }
                count += 2;
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


}
