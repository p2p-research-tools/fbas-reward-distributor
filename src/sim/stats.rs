use crate::{rank::round_to_three_places, Score};

/// Expects a list of approximations and one of the truth values
/// Returns a tuple of absolute error values in the order of the name of the function
pub fn mean_med_pctg_errors(approx: &[Score], exact: &[Score]) -> (f64, f64, f64) {
    let mean = mean_abs_error(approx, exact);
    let median = median_abs_error(approx, exact);
    let percentage = mean_abs_pctg_error(approx, exact);
    (mean, median, percentage)
}

fn mean_abs_error(approximation: &[Score], truth: &[Score]) -> f64 {
    let mut mean_error = 0.0;
    assert!(approximation.len() == truth.len());
    for (i, value) in approximation.iter().enumerate() {
        let error: f64 = (truth[i] - value).abs();
        mean_error += error;
    }
    mean_error /= approximation.len() as f64;
    mean_error
}

fn median_abs_error(approximation: &[f64], truth: &[f64]) -> f64 {
    let mut abs_diff_pred_true: Vec<f64> = Vec::default();
    assert!(approximation.len() == truth.len());
    for (i, value) in approximation.iter().enumerate() {
        let abs_diff: f64 = (value - truth[i]).abs();
        abs_diff_pred_true.push(abs_diff);
    }
    abs_diff_pred_true.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = abs_diff_pred_true.len() / 2;
    abs_diff_pred_true[mid]
}

// The idea of this metric is to be sensitive to relative errors. It is for example not changed by
// a global scaling of the target variable.
fn mean_abs_pctg_error(approximation: &[f64], truth: &[f64]) -> f64 {
    let epsilon: f64 = f64::EPSILON; //  is an arbitrary small yet strictly positive number to avoid undefined results when y is zero
    let mut average_percentage_error = 0.0;
    for (i, value) in approximation.iter().enumerate() {
        let abs_diff: f64 = (truth[i] - value).abs();
        let max = epsilon.max(truth[i]);
        average_percentage_error += abs_diff / max;
    }
    round_to_three_places((1.0 / (approximation.len() as f64)) * average_percentage_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    // test cases from https://scikit-learn.org/stable/modules/model_evaluation.html
    #[test]
    fn mean_error() {
        let prediction = vec![3.0, -0.5, 2.0, 7.0];
        let truth = vec![2.5, 0.0, 2.0, 8.0];
        let actual = mean_abs_error(&prediction, &truth);
        let expected = 0.5;
        assert_eq!(expected, actual);
    }

    #[test]
    fn median_error() {
        let prediction = vec![2.5, 0.0, 2.0, 8.0];
        let truth = vec![3.0, -0.5, 2.0, 7.0];
        let actual = median_abs_error(&prediction, &truth);
        let expected = 0.5;
        assert_eq!(expected, actual);
    }
    #[test]
    fn percentage_error() {
        let truth = vec![1.0, 10.0, 1e6];
        let prediction = vec![0.9, 15.0, 1.2e6];
        let actual = mean_abs_pctg_error(&prediction, &truth);
        let expected = 0.266;
        assert_eq!(expected, actual);
    }
}
