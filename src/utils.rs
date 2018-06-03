use std::cmp::Ordering;

/// This function performs a binary search in a **sorted** slice
/// and returns the index of the closest element.
/// 
/// Returns None if it encounter an invalid value (NAN) or an empty vector
pub fn search_closest_idx(values: &[f64], target_value: &f64) -> Option<usize> {
    // this function performs a dichotomie search on `target_value` inside `values`
    // basically like "the price is right".

    // avoid invalid inputs
    if values.len() == 0 || target_value.is_nan() {
        return None;
    }
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
		match (values[idx]).partial_cmp(target_value) {
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
