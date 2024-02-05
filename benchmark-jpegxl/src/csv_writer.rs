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
    pub file_size_ratio: f64,
    pub raw_size_ratio: f64,
    pub mse: f64,
    pub psnr: f64,
    pub ssim: f64,
    pub ms_ssim: f64,
    pub butteraugli: f64,
    pub butteraugli_pnorm: f64,
    pub ssimulacra2: f64,
}

pub struct ComparisonResultCSVWriter {}

impl ComparisonResultCSVWriter {
    pub fn new() -> Self {
        ComparisonResultCSVWriter {}
    }
}

impl CSVWriter<ComparisonResult> for ComparisonResultCSVWriter {
    fn write_csv(
        &self,
        data: &Vec<ComparisonResult>,
        file_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        println!("Writing {} records to {}", data.len(), file_name);
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
                &record.file_size_ratio.to_string(),
                &record.raw_size_ratio.to_string(),
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
        println!("Done writing {:?} to {}", data[0], file_name);
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        println!("Writing header to {}", file_name);
        let path = std::path::Path::new(file_name);
        if path.exists() && path.metadata()?.len() > 0 {
            println!("File {} already exists", file_name);
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

pub struct ImageFileDataCSVWriter {}

impl ImageFileDataCSVWriter {
    pub fn new() -> Self {
        ImageFileDataCSVWriter {}
    }
}

impl CSVReader<ImageFileData> for ImageFileDataCSVWriter {
    fn read_csv(&self, file_name: &str) -> Result<Vec<ImageFileData>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_name)?;
        let mut data = Vec::new();
        println!(
            "Reading {} records from {}",
            rdr.records().count(),
            file_name
        );
        for result in rdr.records() {
            let record = result?;
            let image_file_data = ImageFileData {
                image_name: record[0].to_string(),
                test_set: record[1].to_string(),
                file_path: record[2].to_string(),
                width: record[3].parse::<u32>()?,
                height: record[4].parse::<u32>()?,
                file_size: record[5].parse::<usize>()?,
                raw_size: record[6].parse::<usize>()?,
                color_space: record[7].to_string().into(),
                file_format: record[8].to_string().into(),
                jxl_orig_image_name: record[9].to_string().into(),
                jxl_distance: record[10].parse::<f32>().unwrap().into(),
                jxl_effort: record[11].parse::<u32>().unwrap().into(),
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
                test_set: record[1].to_string(),
                file_path: record[2].to_string(),
                width: record[3].parse::<u32>()?,
                height: record[4].parse::<u32>()?,
                file_size: record[5].parse::<usize>()?,
                raw_size: record[6].parse::<usize>()?,
                color_space: record[7].to_string().into(),
                file_format: record[8].to_string().into(),
                jxl_orig_image_name: record[9].to_string().into(),
                jxl_distance: record[10].to_string().into(),
                jxl_effort: record[11].to_string().into(),
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
                test_set: record[1].to_string(),
                file_path: record[2].to_string(),
                width: record[3].parse::<u32>()?,
                height: record[4].parse::<u32>()?,
                file_size: record[5].parse::<usize>()?,
                raw_size: record[6].parse::<usize>()?,
                color_space: record[7].to_string().into(),
                file_format: record[8].to_string().into(),
                jxl_orig_image_name: record[9].to_string().into(),
                jxl_distance: record[10].to_string().into(),
                jxl_effort: record[11].to_string().into(),
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

impl CSVWriter<ImageFileData> for ImageFileDataCSVWriter {
    fn write_csv(&self, data: &Vec<ImageFileData>, file_name: &str) -> Result<(), Box<dyn Error>> {
        println!("Writing {} records to {}", data.len(), file_name);
        let file = OpenOptions::new().append(true).open(file_name)?;
        let mut wtr = csv::Writer::from_writer(file);
        for record in data {
            wtr.write_record(&[
                &record.image_name,
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
        println!("Done writing {:?} to {}", data[0], file_name);
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        println!("Writing header to {}", file_name);
        let path = std::path::Path::new(file_name);
        if path.exists() && path.metadata()?.len() > 0 {
            println!("File {} already exists", file_name);
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
