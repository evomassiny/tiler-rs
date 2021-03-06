//!
//! A netCDF to tile converter.
//! This crate provides basic methods to export a netCDF gridded dataset
//! into PNG tiles, suited for interactive web maps.
//!
//! # Usage
//!
//! In order to export a netCDF file into PNGs you need to create 2 objects:
//!
//! * A `Dataset` which hold informations about a netCDF file, such as:
//!     * the *longitude* and *latitude* dimension name,
//!     * the main *variable name*, (the one you which to render),
//!     * the netCDF file *path*.
//! * A `Renderer` which provides convenient functions to render a `Dataset` 
//! into tile images.
//!
//! Here is a simple example: 
//!
//! ```
//! // Create a dataset from a netCDF file
//! let dataset = tiler::Dataset::new(
//!     "latitude",         // Name of the latitude dimension
//!     "longitude",        // Name of the longitude dimension
//!     "wind_magnitude",   // Name of the latitude dimension
//!     "./examples_data/wind_magnitude_reduced.nc" // Path to a dataset
//! ).unwrap();
//! 
//! // Create a renderer
//! let renderer = tiler::Renderer::from_dataset(
//!     dataset,        // input dataset
//!     tiler::Scale::Linear { // Use a linear range of color
//!         min: 0., // minimum value of the colorbar
//!         max: 20.  // maximum value of the colorbar 
//!     },
//!     tiler::ColorMap::RdYlBu_r   // The ColorMap you want to use, (Red Yellow Blue)
//! ).unwrap();
//! 
//! // create a Tile Struct
//! let tile = tiler::Tile {x: 0, y: 0, z: 0 };
//! 
//! // render it into an image (TileImg)
//! if let Ok(img) = renderer.render_tile(&tile) {
//!     // save it as a png file
//!     img.save("./tile_0_0_0.png");
//! }
//! ```
//!
extern crate netcdf;
extern crate image;
extern crate regex;
mod tile;
mod colormap;
mod scale;
mod tiledata;
mod dataset;
mod renderer;
mod utils;
pub use tiledata::TileData;
pub use renderer::{Renderer,ImgTile};
pub use dataset::Dataset;
pub use colormap::{ColorMap,CustomColormap};
pub use tile::Tile;
pub use scale::*;

