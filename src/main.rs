extern crate netcdf;
extern crate image;
mod tile;
mod tiledata;
mod dataset;
mod renderer;
use renderer::Renderer;
use dataset::Dataset;
use tile::Tile;
use std::fs::create_dir_all;

fn main() {
    let cache_path = "./cache/";
    // The dataset must :
    //  - use the WGS84 coordinate system
    //  - have its longitude and latitiude sorted in ascending order
    //  - have its main variable expressed in (lat, lon)
    let dataset_path = "./dataset/wind_magnitude_reduced.nc";

    println!("Openning dataset");
    let dataset = Dataset::new(
        "latitude", 
        "longitude",
        "wind_magnitude",
        dataset_path
    ).unwrap();

    let (value_min, value_max) = (0., 12.);
    println!("Creating a grayscale renderer");
    let renderer = Renderer::from_dataset(dataset, value_min, value_max).unwrap();

    let mut max: u16 = 2;
    // iter Zoom level
    for z in 1..5 {
        println!("Rendering zoom level {}", &z);
        // iter X tile coordinates
        for x in 0..max {
            let tile_dir = format!("{}/{}/{}", &cache_path, &z, &x);
            match create_dir_all(&tile_dir) {
                Ok(_) => {
                    // iter Y tile coordinates
                    for y in 0..max {
                        // create a Tile using x, y, z
                        let tile = Tile {x: x, y:y, z:z };
                        let tile_path = format!("{}/{}.png", &tile_dir, &y);
                        
                        // render it into a png
                        if let Ok(img) = renderer.render_tile(&tile) {
                            // save it
                            img.save(&tile_path);
                        }
                    }
                },
                Err(_) => { continue; }
            }
        }
        max *= 2;
    }
}
