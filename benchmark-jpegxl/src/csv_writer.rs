use crate::image_reader::ImageFileData;

use std::error::Error;
use std::fs::OpenOptions;

pub trait CSVWriter<T>
where
    T: Sized,
{
    fn write_csv(&self, data: &Vec<T>, file_name: &str) -> Result<(), Box<dyn Error>>;
    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>>;
}

pub trait CSVReader<T>
where
    T: Sized,
{
    fn read_csv(&self, file_name: &str) -> Result<Vec<T>, Box<dyn Error>>;
    fn read_entry(&self, file_name: &str, entry: usize) -> Result<T, Box<dyn Error>>;
    fn find_entry(&self, file_name: &str, column: usize, value: &str) -> Result<T, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub orig_image_name: String,
    pub comp_image_name: String,
    pub distance: f32,
    pub effort: u32,
    pub orig_file_size: u64,
    pub comp_file_size: u64,
    pub orig_raw_size: u64,
    pub comp_raw_size: u64,
    pub comp_file_size_ratio: f64,
    pub raw_file_size_ratio: f64,
    pub mse: f64,
    pub psnr: f64,
    pub ssim: f64,
    pub ms_ssim: f64,
    pub butteraugli: f64,
    pub butteraugli_pnorm: f64,
    pub ssimulacra2: f64,
}

#[derive(Debug, Clone)]
pub struct ComparisonResultDiff {
    pub orig_image_name: String,
    pub comp_image_name: String,
    pub distance: f32,
    pub effort: u32,
    pub diff_orig_file_size: f64,
    pub diff_comp_file_size: f64,
    pub diff_orig_raw_size: f64,
    pub diff_comp_raw_size: f64,
    pub diff_comp_file_size_ratio: f64,
    pub diff_raw_file_size_ratio: f64,
    pub diff_mse: f64,
    pub diff_psnr: f64,
    pub diff_ssim: f64,
    pub diff_ms_ssim: f64,
    pub diff_butteraugli: f64,
    pub diff_butteraugli_pnorm: f64,
    pub diff_ssimulacra2: f64,
}

pub struct ComparisonResultCSV {}

pub struct ComparisonResultDiffCSV {}

impl ComparisonResultCSV {
    pub fn new() -> Self {
        ComparisonResultCSV {}
    }
}

impl ComparisonResultDiffCSV {
    pub fn new() -> Self {
        ComparisonResultDiffCSV {}
    }
}

impl CSVWriter<ComparisonResult> for ComparisonResultCSV {
    fn write_csv(
        &self,
        data: &Vec<ComparisonResult>,
        file_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new().append(true).open(file_name)?;
        let mut wtr = csv::Writer::from_writer(file);
        for record in data {
            wtr.write_record(&[
                &record.orig_image_name,
                &record.comp_image_name,
                &record.distance.to_string(),
                &record.effort.to_string(),
                &record.orig_file_size.to_string(),
                &record.comp_file_size.to_string(),
                &record.orig_raw_size.to_string(),
                &record.comp_raw_size.to_string(),
                &record.comp_file_size_ratio.to_string(),
                &record.raw_file_size_ratio.to_string(),
                &record.mse.to_string(),
                &record.psnr.to_string(),
                &record.ssim.to_string(),
                &record.ms_ssim.to_string(),
                &record.butteraugli.to_string(),
                &record.butteraugli_pnorm.to_string(),
                &record.ssimulacra2.to_string(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new(file_name);
        if path.exists() && path.metadata()?.len() > 0 {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut wtr = csv::Writer::from_path(file_name)?;
        wtr.write_record(&[
            "Original Image Name",
            "Compressed Image Name",
            "Distance",
            "Effort",
            "Original File Size",
            "Compressed File Size",
            "Original Raw Size",
            "Compressed Raw Size",
            "File Size Ratio",
            "Raw Size Ratio",
            "MSE",
            "PSNR",
            "SSIM",
            "MS-SSIM",
            "Butteraugli",
            "Butteraugli 3-Norm", // TODO: Support for multiple butteraugli p-norms?
            "SSIMULACRA2",
        ])?;
        wtr.flush()?;
        Ok(())
    }
}

impl CSVWriter<ComparisonResultDiff> for ComparisonResultDiffCSV {
    fn write_csv(
        &self,
        data: &Vec<ComparisonResultDiff>,
        file_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new().append(true).open(file_name)?;
        let mut wtr = csv::Writer::from_writer(file);
        for record in data {
            wtr.write_record(&[
                &record.orig_image_name,
                &record.comp_image_name,
                &record.distance.to_string(),
                &record.effort.to_string(),
                &record.diff_orig_file_size.to_string(),
                &record.diff_comp_file_size.to_string(),
                &record.diff_orig_raw_size.to_string(),
                &record.diff_comp_raw_size.to_string(),
                &record.diff_comp_file_size_ratio.to_string(),
                &record.diff_raw_file_size_ratio.to_string(),
                &record.diff_mse.to_string(),
                &record.diff_psnr.to_string(),
                &record.diff_ssim.to_string(),
                &record.diff_ms_ssim.to_string(),
                &record.diff_butteraugli.to_string(),
                &record.diff_butteraugli_pnorm.to_string(),
                &record.diff_ssimulacra2.to_string(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new(file_name);
        if path.exists() && path.metadata()?.len() > 0 {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut wtr = csv::Writer::from_path(file_name)?;
        wtr.write_record(&[
            "Original Image Name",
            "Compressed Image Name",
            "Distance",
            "Effort",
            "Diff Original File Size",
            "Diff Compressed File Size",
            "Diff Original Raw Size",
            "Diff Compressed Raw Size",
            "Diff File Size Ratio",
            "Diff Raw Size Ratio",
            "Diff MSE",
            "Diff PSNR",
            "Diff SSIM",
            "Diff MS-SSIM",
            "Diff Butteraugli",
            "Diff Butteraugli 3-Norm", // TODO: Support for multiple butteraugli p-norms?
            "Diff SSIMULACRA2",
        ])?;
        wtr.flush()?;
        Ok(())
    }
}

impl CSVReader<ComparisonResult> for ComparisonResultCSV {
    fn read_csv(&self, file_name: &str) -> Result<Vec<ComparisonResult>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let comparison_result = ComparisonResult {
                orig_image_name: record[0].to_string(),
                comp_image_name: record[1].to_string(),
                distance: record[2].parse::<f32>().unwrap(),
                effort: record[3].parse::<u32>().unwrap(),
                orig_file_size: record[4].parse::<u64>().unwrap(),
                comp_file_size: record[5].parse::<u64>().unwrap(),
                orig_raw_size: record[6].parse::<u64>().unwrap(),
                comp_raw_size: record[7].parse::<u64>().unwrap(),
                comp_file_size_ratio: record[8].parse::<f64>().unwrap(),
                raw_file_size_ratio: record[9].parse::<f64>().unwrap(),
                mse: record[10].parse::<f64>().unwrap(),
                psnr: record[11].parse::<f64>().unwrap(),
                ssim: record[12].parse::<f64>().unwrap(),
                ms_ssim: record[13].parse::<f64>().unwrap(),
                butteraugli: record[14].parse::<f64>().unwrap(),
                butteraugli_pnorm: record[15].parse::<f64>().unwrap(),
                ssimulacra2: record[16].parse::<f64>().unwrap(),
            };
            data.push(comparison_result);
        }
        Ok(data)
    }

    fn read_entry(
        &self,
        file_name: &str,
        entry: usize,
    ) -> Result<ComparisonResult, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let comparison_result = ComparisonResult {
                orig_image_name: record[0].to_string(),
                comp_image_name: record[1].to_string(),
                distance: record[2].parse::<f32>()?,
                effort: record[3].parse::<u32>()?,
                orig_file_size: record[4].parse::<u64>()?,
                comp_file_size: record[5].parse::<u64>()?,
                orig_raw_size: record[6].parse::<u64>()?,
                comp_raw_size: record[7].parse::<u64>()?,
                comp_file_size_ratio: record[8].parse::<f64>()?,
                raw_file_size_ratio: record[9].parse::<f64>()?,
                mse: record[10].parse::<f64>()?,
                psnr: record[11].parse::<f64>()?,
                ssim: record[12].parse::<f64>()?,
                ms_ssim: record[13].parse::<f64>()?,
                butteraugli: record[14].parse::<f64>()?,
                butteraugli_pnorm: record[15].parse::<f64>()?,
                ssimulacra2: record[16].parse::<f64>()?,
            };
            data.push(comparison_result);
            if data.len() > entry {
                break;
            }
        }
        Ok(data[entry].clone())
    }

    fn find_entry(
        &self,
        file_name: &str,
        column: usize,
        value: &str,
    ) -> Result<ComparisonResult, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        for result in rdr.records() {
            let record = result?;
            let comparison_result = ComparisonResult {
                orig_image_name: record[0].to_string(),
                comp_image_name: record[1].to_string(),
                distance: record[2].parse::<f32>()?,
                effort: record[3].parse::<u32>()?,
                orig_file_size: record[4].parse::<u64>()?,
                comp_file_size: record[5].parse::<u64>()?,
                orig_raw_size: record[6].parse::<u64>()?,
                comp_raw_size: record[7].parse::<u64>()?,
                comp_file_size_ratio: record[8].parse::<f64>()?,
                raw_file_size_ratio: record[9].parse::<f64>()?,
                mse: record[10].parse::<f64>()?,
                psnr: record[11].parse::<f64>()?,
                ssim: record[12].parse::<f64>()?,
                ms_ssim: record[13].parse::<f64>()?,
                butteraugli: record[14].parse::<f64>()?,
                butteraugli_pnorm: record[15].parse::<f64>()?,
                ssimulacra2: record[16].parse::<f64>()?,
            };
            if record[column] == value.to_string() {
                return Ok(comparison_result);
            }
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No entry found for {} in column {}", value, column),
        )))
    }
}

pub struct ImageFileDataCSV {}

impl ImageFileDataCSV {
    pub fn new() -> Self {
        ImageFileDataCSV {}
    }
}

impl CSVReader<ImageFileData> for ImageFileDataCSV {
    fn read_csv(&self, file_name: &str) -> Result<Vec<ImageFileData>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let image_file_data = ImageFileData {
                image_name: record[0].to_string(),
                commit: record[1].to_string(),
                test_set: record[2].to_string(),
                file_path: record[3].to_string(),
                width: record[4].parse::<u32>()?,
                height: record[5].parse::<u32>()?,
                file_size: record[6].parse::<usize>()?,
                raw_size: record[7].parse::<usize>()?,
                color_space: record[8].to_string().into(),
                file_format: record[9].to_string().into(),
                jxl_orig_image_name: record[10].to_string().into(),
                jxl_distance: record[11].parse::<f32>().unwrap().into(),
                jxl_effort: record[12].parse::<u32>().unwrap().into(),
            };
            data.push(image_file_data);
        }
        Ok(data)
    }

    fn read_entry(&self, file_name: &str, entry: usize) -> Result<ImageFileData, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let image_file_data = ImageFileData {
                image_name: record[0].to_string(),
                commit: record[1].to_string(),
                test_set: record[2].to_string(),
                file_path: record[3].to_string(),
                width: record[4].parse::<u32>()?,
                height: record[5].parse::<u32>()?,
                file_size: record[6].parse::<usize>()?,
                raw_size: record[7].parse::<usize>()?,
                color_space: record[8].to_string().into(),
                file_format: record[9].to_string().into(),
                jxl_orig_image_name: record[10].to_string().into(),
                jxl_distance: record[11].to_string().into(),
                jxl_effort: record[12].to_string().into(),
            };
            data.push(image_file_data);
            if data.len() > entry {
                break;
            }
        }
        Ok(data[entry].clone())
    }

    fn find_entry(
        &self,
        file_name: &str,
        column: usize,
        value: &str,
    ) -> Result<ImageFileData, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        for result in rdr.records() {
            let record = result?;
            let image_file_data = ImageFileData {
                image_name: record[0].to_string(),
                commit: record[1].to_string(),
                test_set: record[2].to_string(),
                file_path: record[3].to_string(),
                width: record[4].parse::<u32>()?,
                height: record[5].parse::<u32>()?,
                file_size: record[6].parse::<usize>()?,
                raw_size: record[7].parse::<usize>()?,
                color_space: record[8].to_string().into(),
                file_format: record[9].to_string().into(),
                jxl_orig_image_name: record[10].to_string().into(),
                jxl_distance: record[11].to_string().into(),
                jxl_effort: record[12].to_string().into(),
            };
            if record[column] == value.to_string() {
                return Ok(image_file_data);
            }
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No entry found for {} in column {}", value, column),
        )))
    }
}

impl CSVWriter<ImageFileData> for ImageFileDataCSV {
    fn write_csv(&self, data: &Vec<ImageFileData>, file_name: &str) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new().append(true).open(file_name)?;
        let mut wtr = csv::Writer::from_writer(file);
        for record in data {
            wtr.write_record(&[
                &record.image_name,
                &record.commit,
                &record.test_set,
                &record.file_path,
                &record.width.to_string(),
                &record.height.to_string(),
                &record.file_size.to_string(),
                &record.raw_size.to_string(),
                &record.color_space.to_string(),
                &record.file_format.to_string(),
                &record.jxl_orig_image_name.to_string(),
                &record.jxl_distance.to_string(),
                &record.jxl_effort.to_string(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new(file_name);
        if path.exists() && path.metadata()?.len() > 0 {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut wtr = csv::Writer::from_path(file_name)?;
        wtr.write_record(&[
            "Image Name",
            "Commit",
            "Test Set",
            "File Path",
            "Image Width",
            "Image Height",
            "File Size",
            "Raw Image Size",
            "Image Color Space",
            "File Format",
            "JXL Original Image Name",
            "JXL Distance",
            "JXL Effort",
        ])?;
        wtr.flush()?;
        Ok(())
    }
}
