use std::cmp::Ordering;

/// This function performs a binary search in a **sorted** vector
/// and returns the index of the closest element.
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
pub fn search_closest_idx(values: &Vec<f64>, target_value: &f64) -> Option<usize> {
    // avoid invalid inputs
    if values.len() == 0 || target_value.is_nan() {
        return None;
    }
    let mut step: usize = values.len() / 2;
    let mut idx: usize = step;
    // Perform a binary search (i.e dichotomie)
    loop {
        step /= 2;
        match (values[idx]).partial_cmp(target_value) {
            Some(Ordering::Less) => {
                if step > 0 {
                    idx += step;
                } else {
                    // if idx is the vec boundary
                    if idx == values.len() - 1 {
                        return Some(idx);
                    }
                    // Return closest between `values[idx]` and `values[idx+1]`
                    if (values[idx] - target_value).abs() < (values[idx+1] - target_value).abs() {
                        return Some(idx);
                    }
                    return Some(idx + 1);
                }
            },
            Some(Ordering::Greater) => {
                if step > 0 {
                    idx -= step;
                } else {
                    // if idx is the vec boundary
                    if idx == 0 {
                        return Some(idx);
                    }
                    // Return closest between `values[idx]` and `values[idx-1]`
                    if (values[idx] - target_value).abs() < (values[idx-1] - target_value).abs() {
                        return Some(idx);
                    }
                    return Some(idx - 1);
                }
            },
            // return the index if found
            Some(Ordering::Equal) => { return Some(idx); },
            // If values[idx] is NAN, abort
            None => { return None; },
        }
    }
}

#[test]
fn test_binary_search() {
    let values: Vec<f64> = vec![1., 3., 3.5, 5., 5.1, 6., 8., 11.];
    // search a value exactly equals to one of element
    assert_eq!(search_closest_idx(&values, &3_f64), Some(1_usize));
    // search a value not in the set
    assert_eq!(search_closest_idx(&values, &5.2), Some(4_usize));
    // search values outside the set ranges
    assert_eq!(search_closest_idx(&values, &-3.), Some(0_usize));
    assert_eq!(search_closest_idx(&values, &100.), Some(values.len() -1));
}
