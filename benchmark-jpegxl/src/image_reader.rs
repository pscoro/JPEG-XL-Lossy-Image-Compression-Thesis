use serde::{Serialize, Serializer};
use serde_derive::Serialize;

use image::DynamicImage;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;

use jpegxl_rs::decode::{JxlDecoder, Metadata, Pixels};
use jpegxl_rs::decoder_builder;

/// Supported color spaces for image reading.
#[derive(Debug, Clone)]
pub enum ColorType {
    L8,
    La8,
    Rgb8,
    Rgba8,
    L16,
    La16,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
}

impl ColorType {
    /// Returns the short name of the color space.
    pub fn to_string(&self) -> String {
        match self {
            ColorType::L8 => "L8".to_string(),
            ColorType::La8 => "La8".to_string(),
            ColorType::Rgb8 => "Rgb8".to_string(),
            ColorType::Rgba8 => "Rgba8".to_string(),
            ColorType::L16 => "L16".to_string(),
            ColorType::La16 => "La16".to_string(),
            ColorType::Rgb16 => "Rgb16".to_string(),
            ColorType::Rgba16 => "Rgba16".to_string(),
            ColorType::Rgb32F => "Rgb32F".to_string(),
            ColorType::Rgba32F => "Rgba32F".to_string(),
        }
    }

    /// Returns the full name of the color space.
    pub fn full_name(&self) -> String {
        match self {
            ColorType::L8 => "8-bit Luminance".to_string(),
            ColorType::La8 => "8-bit Luminance with Alpha".to_string(),
            ColorType::Rgb8 => "8-bit RGB".to_string(),
            ColorType::Rgba8 => "8-bit RGB with Alpha".to_string(),
            ColorType::L16 => "16-bit Luminance".to_string(),
            ColorType::La16 => "16-bit Luminance with Alpha".to_string(),
            ColorType::Rgb16 => "16-bit RGB".to_string(),
            ColorType::Rgba16 => "16-bit RGB with Alpha".to_string(),
            ColorType::Rgb32F => "32-bit Floating Point RGB".to_string(),
            ColorType::Rgba32F => "32-bit Floating Point RGB with Alpha".to_string(),
        }
    }

    /// Gets the colorspace of specificaly a JXL image.
    ///
    /// # Arguments
    /// * `metadata` - The metadata of the JXL image.
    /// * `pixels` - The pixels of the JXL image.
    ///
    /// # Returns
    /// The colorspace of the JXL image.
    fn get_jxl_color_space(metadata: &Metadata, pixels: &Pixels) -> ColorType {
        // The color space is determined by the number of color channels and the pixel type.
        match metadata.num_color_channels {
            1 => match pixels {
                Pixels::Uint8(_) => ColorType::L8,
                Pixels::Uint16(_) => ColorType::L16,
                _ => panic!("Unknown jxl color space"),
            },
            2 => match pixels {
                Pixels::Uint8(_) => ColorType::La8,
                Pixels::Uint16(_) => ColorType::La16,
                _ => panic!("Unknown jxl color space"),
            },
            3 => match pixels {
                Pixels::Uint8(_) => ColorType::Rgb8,
                Pixels::Uint16(_) => ColorType::Rgb16,
                _ => panic!("Unknown jxl color space"),
            },
            4 => match pixels {
                Pixels::Uint8(_) => ColorType::Rgba8,
                Pixels::Uint16(_) => ColorType::Rgba16,
                _ => panic!("Unknown jxl color space"),
            },
            _ => todo!(),
        }
    }
}

/// Supported image formats for image reading.
#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    JpegXl,
    Png,
    Jpeg,
    Gif,
    WebP,
    Pnm,
    Tiff,
    Tga,
    Dds,
    Bmp,
    Ico,
    Hdr,
    OpenExr,
    Farbfeld,
    Avif,
    Qoi,
    Unsupported,
}

impl ImageFormat {
    /// Returns the short name of the image format.
    pub fn to_string(&self) -> String {
        match self {
            ImageFormat::JpegXl => "jxl".to_string(),
            ImageFormat::Png => "png".to_string(),
            ImageFormat::Jpeg => "jpeg".to_string(),
            ImageFormat::Gif => "gif".to_string(),
            ImageFormat::WebP => "webp".to_string(),
            ImageFormat::Pnm => "ppm".to_string(),
            ImageFormat::Tiff => "tiff".to_string(),
            ImageFormat::Tga => "tga".to_string(),
            ImageFormat::Dds => "dds".to_string(),
            ImageFormat::Bmp => "bmp".to_string(),
            ImageFormat::Ico => "ico".to_string(),
            ImageFormat::Hdr => "hdr".to_string(),
            ImageFormat::OpenExr => "exr".to_string(),
            ImageFormat::Farbfeld => "ff".to_string(),
            ImageFormat::Avif => "avif".to_string(),
            ImageFormat::Qoi => "qoi".to_string(),
            ImageFormat::Unsupported => panic!("Unsupported image format"),
        }
    }

    /// Returns the extension of the image format.
    pub fn as_ext(&self) -> String {
        match self {
            ImageFormat::JpegXl => ".jxl".to_string(),
            ImageFormat::Png => ".png".to_string(),
            ImageFormat::Jpeg => ".jpg".to_string(),
            ImageFormat::Gif => ".gif".to_string(),
            ImageFormat::WebP => ".webp".to_string(),
            ImageFormat::Pnm => ".ppm".to_string(),
            ImageFormat::Tiff => ".tiff".to_string(),
            ImageFormat::Tga => ".tga".to_string(),
            ImageFormat::Dds => ".dds".to_string(),
            ImageFormat::Bmp => ".bmp".to_string(),
            ImageFormat::Ico => ".ico".to_string(),
            ImageFormat::Hdr => ".hdr".to_string(),
            ImageFormat::OpenExr => ".exr".to_string(),
            ImageFormat::Farbfeld => ".ff".to_string(),
            ImageFormat::Avif => ".avif".to_string(),
            ImageFormat::Qoi => ".qoi".to_string(),
            ImageFormat::Unsupported => panic!("Unsupported image format"),
        }
    }

    /// Gets the image format from a file name.
    ///
    /// # Arguments
    /// * `file_name` - The name of the file.
    ///
    /// # Returns
    /// The image format of the file.
    pub fn from_file_name(file_name: &str) -> ImageFormat {
        let path = Path::new(file_name);
        let extension = path.extension().unwrap().to_str().unwrap();
        match extension {
            "jxl" => ImageFormat::JpegXl,
            "png" => ImageFormat::Png,
            "jpeg" => ImageFormat::Jpeg,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::WebP,
            "ppm" => ImageFormat::Pnm,
            "tiff" => ImageFormat::Tiff,
            "tga" => ImageFormat::Tga,
            "dds" => ImageFormat::Dds,
            "bmp" => ImageFormat::Bmp,
            "ico" => ImageFormat::Ico,
            "hdr" => ImageFormat::Hdr,
            "exr" => ImageFormat::OpenExr,
            "ff" => ImageFormat::Farbfeld,
            "avif" => ImageFormat::Avif,
            "qoi" => ImageFormat::Qoi,
            _ => ImageFormat::Unsupported,
        }
    }
}

/// Hacky optional f32 wrapper for serialization.
#[derive(Clone, Copy)]
pub struct JXLf32(Option<f32>);

impl JXLf32 {
    /// Creates a new JXLf32.
    ///
    /// # Arguments
    /// * `value` - The value of the JXLf32 as an optional f32.
    ///
    /// # Returns
    /// The JXLf32.
    pub fn new(value: Option<f32>) -> JXLf32 {
        JXLf32(value)
    }

    /// Converts the JXLf32 to a string.
    /// If the value is None, an empty string is returned.
    ///
    /// # Returns
    /// The JXLf32 as a string.
    pub fn to_string(&self) -> String {
        match self.0 {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}


/// Hacky optional u32 wrapper for serialization.
#[derive(Clone, Copy)]
pub struct JXLu32(Option<u32>);

impl JXLu32 {
    /// Creates a new JXLu32.
    ///
    /// # Arguments
    /// * `value` - The value of the JXLu32 as an optional u32.
    ///
    /// # Returns
    /// The JXLu32.
    pub fn new(value: Option<u32>) -> JXLu32 {
        JXLu32(value)
    }

    /// Converts the JXLu32 to a string.
    /// If the value is None, an empty string is returned.
    ///
    /// # Returns
    /// The JXLu32 as a string.
    pub fn to_string(&self) -> String {
        match self.0 {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

/// Hacky optional string wrapper for serialization.
#[derive(Clone)]
pub struct JXLString(Option<String>);

impl JXLString {
    /// Creates a new JXLString.
    ///
    /// # Arguments
    /// * `value` - The value of the JXLString as an optional string.
    ///
    /// # Returns
    /// The JXLString.
    pub fn new(value: Option<String>) -> JXLString {
        JXLString(value)
    }

    /// Converts the JXLString to a string.
    /// If the value is None, an empty string is returned.
    ///
    /// # Returns
    /// The JXLString as a string.
    pub fn to_string(&self) -> String {
        match &self.0 {
            Some(value) => value.clone(),
            None => "".to_string(),
        }
    }
}

/// Metadata of an image file.
#[derive(Debug, Clone, Serialize)]
pub struct ImageFileData {
    pub image_name: String,
    pub commit: String,
    pub test_set: String,
    pub file_path: String,
    pub width: u32,
    pub height: u32,
    pub file_size: usize,
    pub raw_size: usize,
    pub color_space: ColorType,
    pub file_format: ImageFormat,
    pub jxl_orig_image_name: JXLString,
    pub jxl_distance: JXLf32,
    pub jxl_effort: JXLu32,
}

/// Reads an image file and extracts its metadata.
pub struct ImageReader {
    pub image: Option<DynamicImage>,
    pub file_data: ImageFileData,
}

impl ImageReader {
    /// Creates a new ImageReader.
    ///
    /// # Arguments
    /// * `file_path` - The path to the image file.
    /// * `commit` - The commit hash of the image file.
    ///
    /// # Returns
    /// The ImageReader.
    pub fn new(file_path: String, commit: String) -> ImageReader {
        let path = Path::new(&file_path);
        
        // Check that extension is supported image format.
        let extension = path.extension().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
        if ImageFormat::from(extension.to_string()) == ImageFormat::Unsupported {
            panic!("Unsupported image format, this should have been caught earlier");
        }

        // Read JXL files separately since the image crate does not support them.
        if extension == "jxl" {
            return ImageReader::read_jxl(file_path, commit);
        }

        // Read the image file with the image crate.
        let image = image::open(&path).unwrap();

        // Create the ImageReader with the given image.
        ImageReader {
            image: Some(image.clone()),
            file_data: ImageFileData {
                image_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                commit,
                test_set: path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                file_path: file_path.clone(),
                width: image.width(),
                height: image.height(),
                file_size: ImageReader::get_file_size(&file_path),
                raw_size: ImageReader::get_raw_size(&file_path),
                color_space: image.color().into(),
                file_format: ImageReader::get_format(&file_path),
                jxl_orig_image_name: JXLString::new(None),
                jxl_distance: JXLf32::new(None),
                jxl_effort: JXLu32::new(None),
            },
        }
    }

    /// Reads a JXL image file and extracts its metadata.
    ///
    /// # Arguments
    /// * `file_path` - The path to the JXL image file.
    /// * `commit` - The commit hash of the JXL image file.
    ///
    /// # Returns
    /// The ImageReader.
    fn read_jxl(file_path: String, commit: String) -> ImageReader {
        let sample = std::fs::read(file_path.clone()).unwrap();

        // Decode the JXL image using the jpegxl_rs JXL decoder.
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let (metadata, pixels) = decoder.decode(&sample).unwrap();

        // Get the file name and extension.
        let path = Path::new(&file_path);
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let extension = path.extension().unwrap().to_str().unwrap();
        if extension != "jxl" {
            panic!("Not a .jxl file");
        }

        let file_name_parts: Vec<&str> = file_name.split("-").collect();
        let orig_image_name = file_name_parts[0..file_name_parts.len() - 2].join("-");

        // Get the distance and effort values from the file name.
        let distance = file_name_parts[file_name_parts.len() - 2].parse::<f32>();
        let distance = match extension {
            "jxl" => match distance {
                Ok(value) => JXLf32::new(Some(value)),
                Err(_) => JXLf32::new(None),
            },
            _ => JXLf32::new(None),
        };

        let effort = file_name_parts
            .last()
            .unwrap()
            .split(".")
            .next()
            .unwrap()
            .parse::<u32>();
        let effort = match extension {
            "jxl" => match effort {
                Ok(value) => JXLu32::new(Some(value)),
                Err(_) => JXLu32::new(None),
            },
            _ => JXLu32::new(None),
        };

        // Create the ImageReader with the given image.
        ImageReader {
            image: None,
            file_data: ImageFileData {
                image_name: file_name.clone(),
                commit,
                test_set: Path::new(&file_path)
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                file_path: file_path.clone(),
                width: metadata.width.clone(),
                height: metadata.height.clone(),
                file_size: ImageReader::get_file_size(&file_path),
                raw_size: ImageReader::get_raw_jxl_size(&file_path),
                color_space: ColorType::get_jxl_color_space(&metadata, &pixels),
                file_format: ImageReader::get_format(&file_path),
                jxl_orig_image_name: JXLString::new(Some(orig_image_name)),
                jxl_distance: distance,
                jxl_effort: effort,
            },
        }
    }

    /// Gets the image format from a file name.
    ///
    /// # Arguments
    /// * `file_path` - The path to the image file.
    ///
    /// # Returns
    /// The image format of the file.
    fn get_format(file_path: &String) -> ImageFormat {
        let path = Path::new(file_path);
        let extension = path.extension().unwrap().to_str().unwrap();
        match extension {
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::WebP,
            "ppm" => ImageFormat::Pnm,
            "tiff" => ImageFormat::Tiff,
            "tga" => ImageFormat::Tga,
            "dds" => ImageFormat::Dds,
            "bmp" => ImageFormat::Bmp,
            "ico" => ImageFormat::Ico,
            "hdr" => ImageFormat::Hdr,
            "exr" => ImageFormat::OpenExr,
            "ff" => ImageFormat::Farbfeld,
            "avif" => ImageFormat::Avif,
            "qoi" => ImageFormat::Qoi,
            _ => ImageFormat::Jpeg,
        }
    }

    /// Gets the size of a file.
    ///
    /// # Arguments
    /// * `file_path` - The path to the file.
    ///
    /// # Returns
    /// The size of the file in bytes as a usize.
    fn get_file_size(file_path: &String) -> usize {
        let metadata = std::fs::metadata(file_path).unwrap();
        let size = metadata.len();
        size as usize
    }

    /// Gets the raw size of a JXL file.
    /// This is done using the image height, width, and bit depth depending on the color space.
    ///
    /// # Arguments
    /// * `file_path` - The path to the JXL file.
    ///
    /// # Returns
    /// The size of the raw image in bytes as a usize.
    fn get_raw_jxl_size(file_path: &String) -> usize {
        let sample = std::fs::read(file_path.clone()).unwrap();
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let (metadata, pixels) = decoder.decode(&sample).unwrap();
        let width = metadata.width;
        let height = metadata.height;
        let bytes_per_pixel = match ColorType::get_jxl_color_space(&metadata, &pixels) {
            ColorType::L8 => 1,
            ColorType::La8 => 2,
            ColorType::Rgb8 => 3,
            ColorType::Rgba8 => 4,
            ColorType::L16 => 2,
            ColorType::La16 => 4,
            ColorType::Rgb16 => 6,
            ColorType::Rgba16 => 8,
            ColorType::Rgb32F => 12,
            ColorType::Rgba32F => 16,
        };
        let size = width * height * bytes_per_pixel;
        size as usize
    }

    /// Gets the raw size of an image file.
    /// Does not support JXL files, use get_raw_jxl_size instead.
    /// This is done using the image height, width, and bit depth depending on the color space.
    ///
    /// # Arguments
    /// * `file_path` - The path to the image file.
    ///
    /// # Returns
    /// The size of the raw image in bytes as a usize.
    fn get_raw_size(file_path: &String) -> usize {
        let image = image::open(&file_path).unwrap();
        let color_space = image.color();
        let width = image.width();
        let height = image.height();
        let bytes_per_pixel = match color_space {
            image::ColorType::L8 => 1,
            image::ColorType::La8 => 2,
            image::ColorType::Rgb8 => 3,
            image::ColorType::Rgba8 => 4,
            image::ColorType::L16 => 2,
            image::ColorType::La16 => 4,
            image::ColorType::Rgb16 => 6,
            image::ColorType::Rgba16 => 8,
            image::ColorType::Rgb32F => 12,
            image::ColorType::Rgba32F => 16,
            _ => panic!("Unsupported color space"),
        };
        let size = width * height * bytes_per_pixel;
        size as usize
    }

    /// Calculates the mean squared error between two images.
    /// The images are compared pixel by pixel.
    /// The mean squared error is the average of the squared differences between the two images.
    ///
    /// # Arguments
    /// * `orig_image_path` - The path to the original image.
    /// * `comp_image_path` - The path to the compressed image.
    ///
    /// # Returns
    /// The mean squared error between the two images as a f64.
    pub fn calculate_mse(orig_image_path: &String, comp_image_path: &String) -> f64 {
        // Read the original and compressed images, assume the compressed image is a JXL image.
        let orig_image = image::open(orig_image_path).unwrap();
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let comp_image = std::fs::read(comp_image_path.clone()).unwrap();
        let (comp_metadata, comp_pixels) = decoder.decode(&comp_image).unwrap();
        let orig_image = match ColorType::get_jxl_color_space(&comp_metadata, &comp_pixels) {
            ColorType::Rgb8 => orig_image.to_rgb8(),
            _ => todo!(),
        };
        let orig_image = orig_image.as_flat_samples();

        // Calculate the mean squared error between the original and compressed images.
        // The images are compared pixel by pixel. Match the correct pixel type.
        let mut mse = 0.0;
        match comp_pixels {
            Pixels::Uint8(comp_pixels) => {
                for i in 0..orig_image.samples.len() {
                    mse += (orig_image.samples[i] as f64 - comp_pixels[i] as f64).powi(2);
                }
            }
            Pixels::Uint16(comp_pixels) => {
                for i in 0..orig_image.samples.len() {
                    mse += (orig_image.samples[i] as f64 - comp_pixels[i] as f64).powi(2);
                }
            }
            Pixels::Float(comp_pixels) => {
                for i in 0..orig_image.samples.len() {
                    mse += (orig_image.samples[i] as f64 - comp_pixels[i] as f64).powi(2);
                }
            }
            Pixels::Float16(comp_pixels) => {
                for i in 0..orig_image.samples.len() {
                    mse += (orig_image.samples[i] as f64 - f64::from(comp_pixels[i])).powi(2);
                }
            }
        }
        mse /= orig_image.samples.len() as f64;
        mse
    }

    /// Calculates the peak signal-to-noise ratio between two images.
    ///
    /// # Arguments
    /// * `mse` - The mean squared error between the two images.
    /// * `max_value` - The maximum possible pixel value.
    ///
    /// # Returns
    /// The peak signal-to-noise ratio between the two images as a f64.
    pub fn calculate_psnr(mse: f64, max_value: f64) -> f64 {
        10.0 * ((max_value * max_value) / mse).log10()
    }
}

impl Serialize for ColorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for ColorType {
    fn from(color_type: String) -> Self {
        match color_type.as_str() {
            "L8" => ColorType::L8,
            "La8" => ColorType::La8,
            "Rgb8" => ColorType::Rgb8,
            "Rgba8" => ColorType::Rgba8,
            "L16" => ColorType::L16,
            "La16" => ColorType::La16,
            "Rgb16" => ColorType::Rgb16,
            "Rgba16" => ColorType::Rgba16,
            "Rgb32F" => ColorType::Rgb32F,
            "Rgba32F" => ColorType::Rgba32F,
            _ => todo!(),
        }
    }
}

impl From<image::ColorType> for ColorType {
    fn from(color_type: image::ColorType) -> Self {
        match color_type {
            image::ColorType::L8 => ColorType::L8,
            image::ColorType::La8 => ColorType::La8,
            image::ColorType::Rgb8 => ColorType::Rgb8,
            image::ColorType::Rgba8 => ColorType::Rgba8,
            image::ColorType::L16 => ColorType::L16,
            image::ColorType::La16 => ColorType::La16,
            image::ColorType::Rgb16 => ColorType::Rgb16,
            image::ColorType::Rgba16 => ColorType::Rgba16,
            image::ColorType::Rgb32F => ColorType::Rgb32F,
            image::ColorType::Rgba32F => ColorType::Rgba32F,
            _ => todo!(),
        }
    }
}

impl Serialize for ImageFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<image::ImageFormat> for ImageFormat {
    fn from(image_format: image::ImageFormat) -> Self {
        match image_format {
            image::ImageFormat::Jpeg => ImageFormat::Jpeg,
            image::ImageFormat::Png => ImageFormat::Png,
            image::ImageFormat::Gif => ImageFormat::Gif,
            image::ImageFormat::WebP => ImageFormat::WebP,
            image::ImageFormat::Pnm => ImageFormat::Pnm,
            image::ImageFormat::Tiff => ImageFormat::Tiff,
            image::ImageFormat::Tga => ImageFormat::Tga,
            image::ImageFormat::Dds => ImageFormat::Dds,
            image::ImageFormat::Bmp => ImageFormat::Bmp,
            image::ImageFormat::Ico => ImageFormat::Ico,
            image::ImageFormat::Hdr => ImageFormat::Hdr,
            image::ImageFormat::OpenExr => ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld => ImageFormat::Farbfeld,
            image::ImageFormat::Avif => ImageFormat::Avif,
            image::ImageFormat::Qoi => ImageFormat::Qoi,
            _ => todo!(),
        }
    }
}

impl From<String> for ImageFormat {
    fn from(image_format: String) -> Self {
        match image_format.as_str() {
            "jxl" => ImageFormat::JpegXl,
            "png" => ImageFormat::Png,
            "jpeg" => ImageFormat::Jpeg,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::WebP,
            "ppm" => ImageFormat::Pnm,
            "tiff" => ImageFormat::Tiff,
            "tga" => ImageFormat::Tga,
            "dds" => ImageFormat::Dds,
            "bmp" => ImageFormat::Bmp,
            "ico" => ImageFormat::Ico,
            "hdr" => ImageFormat::Hdr,
            "exr" => ImageFormat::OpenExr,
            "ff" => ImageFormat::Farbfeld,
            "avif" => ImageFormat::Avif,
            "qoi" => ImageFormat::Qoi,
            _ => ImageFormat::Unsupported,
        }
    }
}

impl From<JXLf32> for f32 {
    fn from(value: JXLf32) -> Self {
        match value.0 {
            Some(value) => value,
            None => 0.0,
        }
    }
}

impl From<f32> for JXLf32 {
    fn from(value: f32) -> Self {
        JXLf32(Some(value))
    }
}

impl From<String> for JXLf32 {
    fn from(value: String) -> Self {
        if value.is_empty() {
            JXLf32(None)
        } else {
            JXLf32(Some(value.parse::<f32>().unwrap()))
        }
    }
}

impl Serialize for JXLf32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Some(value) => serializer.serialize_f32(*value),
            None => serializer.serialize_str(""),
        }
    }
}

impl Display for JXLf32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl Debug for JXLf32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl From<JXLu32> for u32 {
    fn from(value: JXLu32) -> Self {
        match value.0 {
            Some(value) => value,
            None => 0,
        }
    }
}

impl From<u32> for JXLu32 {
    fn from(value: u32) -> Self {
        JXLu32(Some(value))
    }
}

impl From<String> for JXLu32 {
    fn from(value: String) -> Self {
        if value.is_empty() {
            JXLu32(None)
        } else {
            JXLu32(Some(value.parse::<u32>().unwrap()))
        }
    }
}

impl Serialize for JXLu32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Some(value) => serializer.serialize_u32(*value),
            None => serializer.serialize_str(""),
        }
    }
}

impl Display for JXLu32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl Debug for JXLu32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl From<String> for JXLString {
    fn from(value: String) -> Self {
        if value.is_empty() {
            JXLString(None)
        } else {
            JXLString(Some(value))
        }
    }
}

impl Serialize for JXLString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Some(value) => serializer.serialize_str(value),
            None => serializer.serialize_str(""),
        }
    }
}

impl Display for JXLString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl Debug for JXLString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

