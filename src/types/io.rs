use crate::Score;
use csv::{ReaderBuilder, Writer, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::{error::Error, io, path::Path};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct InputDataPoint {
    pub top_tier_size: usize,
    pub run: usize,
}
impl InputDataPoint {
    pub fn from_output_data_point(d: &OutputDataPoint) -> Self {
        Self {
            top_tier_size: d.top_tier_size,
            run: d.run,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDataPoint {
    pub top_tier_size: usize,
    pub run: usize,
    pub noderanks: Vec<Score>,
    pub exact_power_indices: Vec<Score>,
    pub approx_power_indices_10_pow_1: Vec<Score>,
    pub approx_power_indices_10_pow_2: Vec<Score>,
    pub approx_power_indices_10_pow_3: Vec<Score>,
    pub approx_power_indices_10_pow_4: Vec<Score>,
    pub approx_power_indices_10_pow_5: Vec<Score>,
    pub approx_power_indices_10_pow_6: Vec<Score>,
    pub approx_power_indices_10_pow_7: Vec<Score>,
    pub approx_power_indices_10_pow_8: Vec<Score>,
    pub duration_noderank: f64,
    pub duration_exact_power_index: f64,
    pub duration_approx_power_indices_10_pow_1: f64,
    pub duration_approx_power_indices_10_pow_2: f64,
    pub duration_approx_power_indices_10_pow_3: f64,
    pub duration_approx_power_indices_10_pow_4: f64,
    pub duration_approx_power_indices_10_pow_5: f64,
    pub duration_approx_power_indices_10_pow_6: f64,
    pub duration_approx_power_indices_10_pow_7: f64,
    pub duration_approx_power_indices_10_pow_8: f64,
    pub duration_after_mq_approx_power_indices_10_pow_1: f64,
    pub duration_after_mq_approx_power_indices_10_pow_2: f64,
    pub duration_after_mq_approx_power_indices_10_pow_3: f64,
    pub duration_after_mq_approx_power_indices_10_pow_4: f64,
    pub duration_after_mq_approx_power_indices_10_pow_5: f64,
    pub duration_after_mq_approx_power_indices_10_pow_6: f64,
    pub duration_after_mq_approx_power_indices_10_pow_7: f64,
    pub duration_after_mq_approx_power_indices_10_pow_8: f64,
}

pub fn read_csv_from_file(path: &Path) -> Result<Vec<OutputDataPoint>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;
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
    let writer = WriterBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;
    write_csv_via_writer(data_points, writer)
}

pub fn write_csv_to_stdout(
    data_points: impl IntoIterator<Item = impl serde::Serialize>,
) -> Result<(), Box<dyn Error>> {
    let writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());
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
