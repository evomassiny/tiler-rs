# tiler-rs
This simple rust program create Web Mercator Tiles from netCDF datasets.

This is a weekend project, don't expect to much of it.

## Build

* You need to have the libnetcdf installed, on debian-based distro :

```
sudo apt-get install libhdf5-serial-dev netcdf-bin libnetcdf-dev
```

* Then you should be able to build and run it using the rust package manager *cargo*:

```
cargo run
```
This will create tiles from the **dataset/wind_magnitude_reduced.nc** file into *./cache*

* Finally, you can look the exported Tiles in your browser by openning **viewer.html**
