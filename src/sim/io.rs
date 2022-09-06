use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::{error::Error, io, path::Path};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct InputDataPoint {
    pub top_tier_size: usize,
    pub run: usize,
}
impl InputDataPoint {
    pub fn from_perf_data_point(d: &PerfDataPoint) -> Self {
        Self {
            top_tier_size: d.top_tier_size,
            run: d.run,
        }
    }
    pub fn from_error_data_point(d: &ErrorDataPoint) -> Self {
        Self {
            top_tier_size: d.top_tier_size,
            run: d.run,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfDataPoint {
    pub top_tier_size: usize,
    pub run: usize,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDataPoint {
    pub top_tier_size: usize,
    pub run: usize,
    pub mean_abs_error_10_pow_1: f64,
    pub median_abs_error_10_pow_1: f64,
    pub mean_abs_percentage_error_10_pow_1: f64,
    pub mean_abs_error_10_pow_2: f64,
    pub median_abs_error_10_pow_2: f64,
    pub mean_abs_percentage_error_10_pow_2: f64,
    pub mean_abs_error_10_pow_3: f64,
    pub median_abs_error_10_pow_3: f64,
    pub mean_abs_percentage_error_10_pow_3: f64,
    pub mean_abs_error_10_pow_4: f64,
    pub median_abs_error_10_pow_4: f64,
    pub mean_abs_percentage_error_10_pow_4: f64,
    pub mean_abs_error_10_pow_5: f64,
    pub median_abs_error_10_pow_5: f64,
    pub mean_abs_percentage_error_10_pow_5: f64,
    pub mean_abs_error_10_pow_6: f64,
    pub median_abs_error_10_pow_6: f64,
    pub mean_abs_percentage_error_10_pow_6: f64,
    pub mean_abs_error_10_pow_7: f64,
    pub median_abs_error_10_pow_7: f64,
    pub mean_abs_percentage_error_10_pow_7: f64,
    pub mean_abs_error_10_pow_8: f64,
    pub median_abs_error_10_pow_8: f64,
    pub mean_abs_percentage_error_10_pow_8: f64,
}

pub fn read_csv_from_file(path: &Path) -> Result<Vec<PerfDataPoint>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut result = vec![];
    for line in reader.deserialize() {
        result.push(line?);
    }
    Ok(result)
}

pub fn read_error_data_csv_from_file(path: &Path) -> Result<Vec<ErrorDataPoint>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut result = vec![];
    for line in reader.deserialize() {
        result.push(line?);
    }
    Ok(result)
}

pub fn write_csv_to_file(
    data_points: impl IntoIterator<Item = impl serde::Serialize>,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let writer = Writer::from_path(path)?;
    write_csv_via_writer(data_points, writer)
}

pub fn write_csv_to_stdout(
    data_points: impl IntoIterator<Item = impl serde::Serialize>,
) -> Result<(), Box<dyn Error>> {
    let writer = Writer::from_writer(io::stdout());
    write_csv_via_writer(data_points, writer)
}

pub fn write_csv_via_writer(
    data_points: impl IntoIterator<Item = impl serde::Serialize>,
    mut writer: Writer<impl io::Write>,
) -> Result<(), Box<dyn Error>> {
    for data_point in data_points.into_iter() {
        writer.serialize(data_point)?;
        writer.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_from_nonexistent_file_doesnt_panic() {
        let file_path = Path::new("");
        let actual = read_csv_from_file(file_path);
        assert!(actual.is_err());
    }

    #[test]
    fn write_to_nonexistent_file_doesnt_panic() {
        let file_path = Path::new("");
        let mock_data = PerfDataPoint {
            top_tier_size: usize::default(),
            run: usize::default(),
            duration: f64::default(),
        };

        let actual = write_csv_to_file(vec![mock_data], file_path);
        assert!(actual.is_err());
    }
}
