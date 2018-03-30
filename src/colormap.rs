use std::io::{BufRead, BufReader};
use std::fs::File;
use regex::Regex;

// data defining a RdYlBu ColorMap (taken from the matplotlib python module)
const RD_TL_BU_DATA: [[f32; 3]; 11] = [
    [0.6470588235294118 , 0.0                 , 0.14901960784313725],
    [0.84313725490196079, 0.18823529411764706 , 0.15294117647058825],
    [0.95686274509803926, 0.42745098039215684 , 0.2627450980392157 ],
    [0.99215686274509807, 0.68235294117647061 , 0.38039215686274508],
    [0.99607843137254903, 0.8784313725490196  , 0.56470588235294117],
    [1.0                , 1.0                 , 0.74901960784313726],
    [0.8784313725490196 , 0.95294117647058818 , 0.97254901960784312],
    [0.6705882352941176 , 0.85098039215686272 , 0.9137254901960784 ],
    [0.45490196078431372, 0.67843137254901964 , 0.81960784313725488],
    [0.27058823529411763, 0.45882352941176469 , 0.70588235294117652],
    [0.19215686274509805, 0.21176470588235294 , 0.58431372549019611]
];

// data defining a BrBG ColorMap (taken from the matplotlib python module)
const BR_BG_DATA: [[f32; 3]; 11] = [
    [0.32941176470588235,  0.18823529411764706,  0.0196078431372549 ],
    [0.5490196078431373 ,  0.31764705882352939,  0.0392156862745098 ],
    [0.74901960784313726,  0.50588235294117645,  0.17647058823529413],
    [0.87450980392156863,  0.76078431372549016,  0.49019607843137253],
    [0.96470588235294119,  0.90980392156862744,  0.76470588235294112],
    [0.96078431372549022,  0.96078431372549022,  0.96078431372549022],
    [0.7803921568627451 ,  0.91764705882352937,  0.89803921568627454],
    [0.50196078431372548,  0.80392156862745101,  0.75686274509803919],
    [0.20784313725490197,  0.59215686274509804,  0.5607843137254902 ],
    [0.00392156862745098,  0.4                ,  0.36862745098039218],
    [0.0                ,  0.23529411764705882,  0.18823529411764706]
];

/// This struct represents a user defined color map,
/// It should directly map values to colors
pub struct CustomColormap {
    values: Vec<f32>,
    colors: Vec<[u8; 3]>
}
impl CustomColormap {

    /// Create a Custom colormap from a QGis colormap file
    ///
    /// # Caution
    /// It only support interpolated colormap
    pub fn from_qgis_file(file: &str) -> Result<ColorMap, String> {
        let file = File::open(file).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut colors: Vec<[u8; 3]> = Vec::new();
        let mut values: Vec<f32> = Vec::new();
        let mut line = String::new();

        // match a QGIS colormap value
        let data_regex =  Regex::new(
            r"^(?P<value>\d+),(?P<red>\d+),(?P<green>\d+),(?P<blue>\d+),\d+,\d+(\.\d*)?\s*$"
        ).unwrap();
        // Iter line of the file
        while let Ok(bytes_read) = reader.read_line(&mut line) {
            if bytes_read == 0 { break; }
            if let Some(capture) = data_regex.captures(&line) {
                // parse each value + pixel color using a big dirty regex
                let value: f32 = capture.name("value").unwrap().as_str().parse::<f32>().unwrap();
                let red: u8 = capture.name("red").unwrap().as_str().parse::<u8>().unwrap();
                let green: u8 = capture.name("green").unwrap().as_str().parse::<u8>().unwrap();
                let blue: u8 = capture.name("blue").unwrap().as_str().parse::<u8>().unwrap();
                colors.push([red, green, blue]);
                values.push(value);
            }
            line.clear();
        }
        if colors.len() > 0 && colors.len() == values.len() {
            return Ok(ColorMap::Custom(
                Self {
                    colors: colors,
                    values: values
                }
            ));
        }
        Err("Could not parse the QGis file".into())
    }

    /// returns a pixel value, from a dataset value.
    fn value_to_color(&self, value: f32) -> [u8; 3] {
        if self.values.len() == 1 || self.values[0] >= value {
            return self.colors[self.values.len() -1];
        }
        if value >= self.values[self.values.len() -1] {
            return self.colors[self.values.len() -1];
        }
        for i in 0..(self.values.len() - 1) {
            if value == self.values[i] {
                return self.colors[i];
            }
            if value > self.values[i] && value < self.values[i + 1] {
                let mut rgb: [u8; 3] = [0; 3];
                let lower_diff = value - self.values[i];
                let upper_diff = self.values[i+1] - value;
                let diff = self.values[i+1] - self.values[i];
                for j in 0..rgb.len() {
                    rgb[j] = ((
                          self.colors[i][j] as f32 * upper_diff + self.colors[i+1][j] as f32 * lower_diff
                       ) / diff ) as u8;
                }
                return rgb;
            }
        }
        [22; 3]
    }
}

/// Defines values to color association.
///
/// By convention, a `Foo_r` Colormap Variant represents the `Foo` colormap reversed.
/// 
#[allow(non_camel_case_types)]
pub enum ColorMap {
    /// black to gray colorMap
    Grayscale,
    /// Red Yellow Blue colorMap
    RdYlBu,
    /// Reversed Red Yellow Blue ColorMap
    RdYlBu_r,
    /// Brown to Green
    BrBG,
    /// Brown to Green, reversed
    BrBG_r,
    /// User defined
    Custom(CustomColormap),
}



fn value_to_grayscale(value: f32) -> [u8; 3] {
    let gray = ((value * 255.) % 255.) as u8;
    [gray, gray, gray]
}

/**
 * Returns pixel colors from a value (between 0 and 1),
 * It linearly interpolate the color between the colors defined in `data`
 */
fn value_to_color(value: f32, data: &[[f32; 3]], reverse: bool) -> [u8; 3] {
    // reverse the value if asked
    let scaled: f32 = if reverse {
        (1. - value) * ((data.len() -1) as f32)
    } else {
        value * ((data.len() -1) as f32)
    };
    // get the minimum indice needed for the interpolation
    let idx = scaled.floor() as usize;
    let weight = scaled % 1.;

    let mut rgb: [u8; 3] = [0; 3];
    if idx == data.len() -1 {
        // don't interpolate max values
        for i in 0..rgb.len() {
            rgb[i] = (data[data.len() -1][i] * 255.) as u8; 
        }
    } else {
        // perform the interpolation for each pixel color (RGB)
        for i in 0..rgb.len() {
            // this is basically a weighted mean
            rgb[i] = ((data[idx][i] * (1. - weight) + weight * data[idx + 1][i]) * 255.) as u8;
        }
    }
    rgb
}

/**
 * Returns a pixel color from a [0; 1] f32 value
 * and a ColorMap variant
 */
pub fn rgb(value: f32, color_map: &ColorMap) -> [u8; 3] {
    match *color_map {
        ColorMap::Grayscale => { value_to_grayscale(value) },
        ColorMap::RdYlBu => { 
            value_to_color(value, &RD_TL_BU_DATA, false)
        },
        ColorMap::RdYlBu_r => {
            value_to_color(value, &RD_TL_BU_DATA, true)
        },
        ColorMap::BrBG => { 
            value_to_color(value, &BR_BG_DATA, false)
        },
        ColorMap::BrBG_r => {
            value_to_color(value, &BR_BG_DATA, true)
        },
        ColorMap::Custom(ref cmap) => {
            cmap.value_to_color(value)
        },
    }
}

#[test]
fn test_colormap_interpolation() {
    let data: [[f32; 3]; 4] = [
        [0., 0., 0.],
        [1./3., 1./3., 1./3.],
        [2./3., 2./3., 2./3.],
        [1., 1., 1.],
    ];
    // test interpolation for 0, 0.5 and 1
    assert_eq!(value_to_color(0., &data, false), [0u8; 3]);
    assert_eq!(value_to_color(1., &data, false), [255u8; 3]);
    assert_eq!(value_to_color(0.5, &data, false), [(255 / 2) as u8; 3]);
    
    // test interpolation for 0, 0.5 and 1 with revesed colormap
    assert_eq!(value_to_color(0., &data, true), [255u8; 3]);
    assert_eq!(value_to_color(1., &data, true), [0u8; 3]);
    assert_eq!(value_to_color(0.5, &data, true), [(255 / 2) as u8; 3]);
}

#[test]
/// Parse a sample QGis colormap file
fn qgis_colormap_parser() {
    let colormap_path = "./examples_data/qgis_colormap.txt";
    let colormap = CustomColormap::from_qgis_file(
        colormap_path
    ).unwrap();
    let expected_values = vec![
        0.,
        1000.,
        2000.,
        3000.,
        4000.,
        5000.,
        6000.,
    ];
    match colormap {
        ColorMap::Custom(cmap) => {
            assert_eq!(cmap.colors.len(), expected_values.len());
            assert_eq!(cmap.values, expected_values);
        }
        _ => { panic!("QGis colormap oarsing failed") }
    }
}
