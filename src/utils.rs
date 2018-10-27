use std::cmp::Ordering;

/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element.
/// (it can handle both ascending and descending order.)
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
pub fn search_closest_idx(values: &[f64], target_value: f64) -> Option<usize> {
    // avoid invalid inputs
    if values.len() == 0 || target_value.is_nan() { return None; } 
    if values.len() == 1 { return Some(0); } 
    if values[0] < values[values.len() -1] {
        return search_closest_idx_asc(values, target_value);
    }
    return search_closest_idx_desc(values, target_value);
}
/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element.
/// Only handles **ASCENDING** order.
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
fn search_closest_idx_asc(values: &[f64], target_value: f64) -> Option<usize> {
    // this function performs a dichotomie search on `target_value` inside `values`
    // basically like "the price is right".

    let max_idx = values.len() -1;
    // size of the dichotomie interval
    let mut step: usize = values.len() / 2;
    // current index
    let mut idx: usize = step;
    // Perform a binary search (i.e dichotomie)
	loop {
        // update the interval length
        // we add one to handle odd values
        step = (1 + step) / 2;
		match (values[idx]).partial_cmp(&target_value) {
            // increment `idx` of `step`
			Some(Ordering::Less) => {
                // avoid out-of-bound index 
                idx = max_idx.min(idx + step);
            },
            // decrement `idx` of `step`
			Some(Ordering::Greater) => { 
                // avoid out-of-bound index and overflow
                idx = if idx > step { idx - step } else { 0 };
            },
			// return the exact index if found
			Some(Ordering::Equal) => return Some(idx),
			// If values[idx] is NAN, abort
			None => return None,
		}
        // If step == 1, we can't get any better 
        // EXIT the loop
        if step == 1 {
            // Return closest between `values[idx]` and `values[idx+1]`
            if idx < max_idx && (values[idx] - target_value).abs() > (values[idx+1] - target_value).abs() {
                return Some(idx+1);
            }
            // Return closest between `values[idx]` and `values[idx-1]`
            if idx > 0 && (values[idx] - target_value).abs() > (values[idx-1] - target_value).abs() {
                return Some(idx -1);
            }
            return Some(idx);
        }
    }
}

/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element.
/// Only handles **DESCENDING** order.
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
fn search_closest_idx_desc(values: &[f64], target_value: f64) -> Option<usize> {
    let max_idx = values.len() -1;
    let mut step: usize = values.len() / 2;
    let mut idx: usize = step;
	loop {
        step = (1 + step) / 2;
		match (values[idx]).partial_cmp(&target_value) {
			Some(Ordering::Less) => {
                idx = if idx > step { idx - step } else { 0 };
            },
			Some(Ordering::Greater) => { 
                idx = max_idx.min(idx + step);
            },
			Some(Ordering::Equal) => return Some(idx),
			None => return None,
		}
        // If step == 1, we can't get any better EXIT the loop
        if step == 1 {
            // Return closest between `values[idx]` and `values[idx+1]`
            if idx < max_idx && (values[idx] - target_value).abs() > (values[idx+1] - target_value).abs() {
                return Some(idx+1);
            }
            // Return closest between `values[idx]` and `values[idx-1]`
            if idx > 0 && (values[idx] - target_value).abs() > (values[idx-1] - target_value).abs() {
                return Some(idx -1);
            }
            return Some(idx);
        }
    }
}

/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element **inferior** than target_value.
/// (it can handle both ascending and descending order.)
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
pub fn search_closest_idx_below(values: &[f64], target_value: f64) -> Option<usize> {
    match  search_closest_idx(values, target_value) {
        Some(idx) => {
            if values[idx] > target_value {
                if idx > 0 && values[idx -1] < values[idx] {
                    // ascending order
                    return Some(idx -1);
                } 
                if idx < values.len() -1 && values[idx] > values[idx +1] {
                    // descending order
                    return Some(idx +1);
                }
            } 
            return Some(idx);
        },
        None => {}
    }
    return None;
}

/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element **superior** than target_value.
/// (it can handle both ascending and descending order.)
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
pub fn search_closest_idx_over(values: &[f64], target_value: f64) -> Option<usize> {
    match  search_closest_idx(values, target_value) {
        Some(idx) => {
            if values[idx] < target_value {
                if idx < values.len() -1 && values[idx +1] > values[idx] {
                    // ascending order
                    return Some(idx +1);
                } 
                if idx > 0 && values[idx -1] > values[idx]{
                    // descending order
                    return Some(idx -1);
                }
            } 
            return Some(idx);
        },
        None => {}
    }
    return None;
}

#[test]
fn test_binary_search() {
    let asc_values: Vec<f64> = vec![1., 3., 3.5, 5., 5.1, 6., 8., 11.];
    // search a value exactly equals to one of element
    assert_eq!(search_closest_idx(&asc_values, 3_f64), Some(1_usize));
    // search a value not in the set
    assert_eq!(search_closest_idx(&asc_values, 5.2), Some(4_usize));
    // search asc_values outside the set ranges
    assert_eq!(search_closest_idx(&asc_values, -3.), Some(0_usize));
    assert_eq!(search_closest_idx(&asc_values, 100.), Some(asc_values.len() -1));
    // search values in a descending order array
    let desc_values: Vec<f64> = vec![999., 455., 100., 1., -89.];
    assert_eq!(search_closest_idx(&desc_values, 400.), Some(1));
    assert_eq!(search_closest_idx(&desc_values, -100.), Some(4));
}

#[test]
fn test_binary_search_below() {
    let asc_values: Vec<f64> = vec![1., 2., 3., 4.];
    assert_eq!(search_closest_idx_below(&asc_values, 2.9_f64), Some(1_usize));
    assert_eq!(search_closest_idx_below(&asc_values, 0.5), Some(0_usize));
    let desc_values: Vec<f64> = vec![999., 455., 100., 1., -89.];
    assert_eq!(search_closest_idx_below(&desc_values, 850.), Some(1));
    assert_eq!(search_closest_idx_below(&desc_values, -100.), Some(4));
}

#[test]
fn test_binary_search_over() {
    let asc_values: Vec<f64> = vec![1., 2., 3., 4.];
    assert_eq!(search_closest_idx_over(&asc_values, 2.1_f64), Some(2_usize));
    assert_eq!(search_closest_idx_over(&asc_values, 4.9_f64), Some(3_usize));
    let desc_values: Vec<f64> = vec![999., 455., 100., 1., -89.];
    assert_eq!(search_closest_idx_over(&desc_values, 850.), Some(0));
    assert_eq!(search_closest_idx_over(&desc_values, 1100.), Some(0));
}
