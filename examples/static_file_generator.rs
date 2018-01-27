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
    println!("Creating a RdYlBu_r renderer");
    let renderer = tiler::Renderer::from_dataset(
        dataset,        // input dataset
        tiler::Scale::Linear { // Use a linear range of color
            min: value_min, // minimum value of the colorbar
            max: value_max  // maximum value of the colorbar 
        },
        tiler::ColorMap::RdYlBu_r   // Red Yellow Blue colormap
    ).unwrap();

    let mut max: u16 = 2;
    // iter Zoom level
    for z in 0..5 {
        println!("Rendering zoom level {}", &z);
        // iter X tile coordinates
        for x in 0..max {
            let tile_dir = format!("{}/{}/{}", &cache_path, &z, &x);
            match create_dir_all(&tile_dir) {
                Ok(_) => {
                    // iter Y tile coordinates
                    for y in 0..max {
                        // create a Tile using x, y, z
                        let tile = tiler::Tile {x: x, y:y, z:z };
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
    println!("You show see the result by opening ./examples_data/viewer.html with your browser.");
}
