use crate::image_reader::ImageFileData;

use std::error::Error;

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
        let mut wtr = csv::Writer::from_path(file_name)?;
        for d in data {
            wtr.serialize(d)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn write_csv_header(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        println!("Writing header to {}", file_name);
        let path = std::path::Path::new(file_name);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut wtr = csv::Writer::from_path(file_name)?;
        wtr.write_record(&[
            "File Path",
            "Image Width",
            "Image Height",
            "File Size",
            "Raw Image Size",
            "File Format",
            "Image Color Space",
        ])?;
        wtr.flush()?;
        Ok(())
    }
}
