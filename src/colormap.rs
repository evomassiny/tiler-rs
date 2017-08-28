// data defining a RdYlBu ColorMap (taken from the matplotlib python
// module)
const RdYlBu_data: [[f32; 3]; 11] = [
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

/// Defines a values to color association
#[allow(non_camel_case_types)]
pub enum ColorMap {
    /// black to gray colorMap
    Grayscale,
    /// Red Yellow Blue colorMap
    RdYlBu,
    /// Reversed Red Yellow Blue ColorMap
    RdYlBu_r,
}

fn value_to_grayscale(value: f32) -> [u8; 3] {
    let gray = ((value * 255.) % 255.) as u8;
    [gray, gray, gray]
}
/**
 * Returns pixel colors from a value (between 0 and 1),
 * It linearly interpolate the color between the colors defindes in `data`
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
        // perform the interpolation
        for i in 0..rgb.len() {
            rgb[i] = ( ( data[idx][i] * (1. - weight) + weight * data[idx + 1][i] ) * 255.) as u8;
        }
    }
    rgb
}

/**
 * Turns a [0; 1] f32 value into pixel colors,
 * Using the RdYlBu ColorMap
 */
#[allow(non_camel_case_types)]
fn value_to_RdYlBu(value: f32) -> [u8; 3] {
    return value_to_color(value, &RdYlBu_data, false);
}

/**
 * Turns a [0; 1] f32 value into pixel colors,
 * Using the reversed RdYlBu ColorMap
 */
#[allow(non_camel_case_types)]
fn value_to_RdYlBu_r(value: f32) -> [u8; 3] {
    return value_to_color(value, &RdYlBu_data, true);
}

/**
 * Returns a pixel color from a [0; 1] f32 value
 * and a ColorMap variant
 */
pub fn rgb(value: f32, color_map: &ColorMap) -> [u8; 3] {
    match *color_map {
        ColorMap::Grayscale => { value_to_grayscale(value) },
        ColorMap::RdYlBu => { value_to_RdYlBu(value) },
        ColorMap::RdYlBu_r => { value_to_RdYlBu_r(value) },
    }
}
