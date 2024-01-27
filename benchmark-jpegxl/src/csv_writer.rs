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

pub struct ImageFileDataCSVWriter {}

impl ImageFileDataCSVWriter {
    pub fn new() -> Self {
        ImageFileDataCSVWriter {}
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
            "JXL Distance",
            "JXL Effort",
        ])?;
        wtr.flush()?;
        Ok(())
    }
}
