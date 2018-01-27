extern crate tiler;
use std::fs::create_dir_all;

fn main() {
    let cache_path = "./examples_data/cache/";
    // The dataset must :
    //  - use the WGS84 coordinate system
    //  - have its longitude and latitiude sorted in ascending order
    //  - have its main variable expressed in (lat, lon)
    let dataset_path = "./examples_data/wind_magnitude_reduced.nc";

    println!("Openning dataset {}", &dataset_path);
    let dataset = tiler::Dataset::new(
        "latitude", 
        "longitude",
        "wind_magnitude",
        dataset_path
    ).unwrap();

    let (value_min, value_max) = (0., 20.);
    println!("Creating a BrBG renderer");
    let renderer = tiler::Renderer::from_dataset(
        dataset,        // input dataset
        tiler::Scale::Linear { // Use a linear range of color
            min: value_min, // minimum value of the colorbar
            max: value_max  // maximum value of the colorbar 
        },
        tiler::ColorMap::BrBG   // Brown to Green
    ).unwrap();

    let tile = tiler::Tile {x: 0, y: 0, z: 0 };

    // recursively creates images until zoom 6
    if let Ok(tile_imgs) = renderer.render_n_level_tile(&tile, 6) {
        for tile_img in tile_imgs {
            let tile_dir = format!("{}/{}/{}", &cache_path, &tile_img.z, &tile_img.x);
            if let Ok(_) =  create_dir_all(&tile_dir) {
                let tile_path = format!("{}/{}.png", &tile_dir, &tile_img.y);
                tile_img.save(&tile_path);
            }
        }
    }
    println!("You show see the result by opening ./examples_data/viewer.html with your browser.");
}

