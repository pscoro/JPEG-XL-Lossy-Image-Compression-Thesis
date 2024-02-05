use serde::{Serialize, Serializer};
use serde_derive::Serialize;

use image::DynamicImage;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;

use jpegxl_rs::decode::{JxlDecoder, Metadata, Pixels};
use jpegxl_rs::decoder_builder;

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

impl ColorType {
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

    fn get_jxl_color_space(metadata: &Metadata, pixels: &Pixels) -> ColorType {
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

#[derive(Debug, Clone)]
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

impl Serialize for ImageFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl ImageFormat {
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
#[derive(Clone, Copy)]
pub struct JXLf32(Option<f32>);

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

impl JXLf32 {
    pub fn new(value: Option<f32>) -> JXLf32 {
        JXLf32(value)
    }

    pub fn to_string(&self) -> String {
        match self.0 {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct JXLu32(Option<u32>);

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

impl JXLu32 {
    pub fn new(value: Option<u32>) -> JXLu32 {
        JXLu32(value)
    }

    pub fn to_string(&self) -> String {
        match self.0 {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct JXLString(Option<String>);

impl JXLString {
    pub fn new(value: Option<String>) -> JXLString {
        JXLString(value)
    }

    pub fn to_string(&self) -> String {
        match &self.0 {
            Some(value) => value.clone(),
            None => "".to_string(),
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

#[derive(Debug, Clone, Serialize)]
pub struct ImageFileData {
    pub image_name: String,
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

pub struct ImageReader {
    pub image: Option<DynamicImage>,
    pub file_data: ImageFileData,
}

impl ImageReader {
    pub fn new(file_path: String) -> ImageReader {
        println!("Reading image from {}", file_path);
        let path = Path::new(&file_path);
        let extension = path.extension().unwrap().to_str().unwrap();
        println!("Extension: {}", extension);
        if extension == "jxl" {
            return ImageReader::read_jxl(file_path);
        }
        let image = image::open(&file_path).unwrap();

        ImageReader {
            image: Some(image.clone()),
            file_data: ImageFileData {
                image_name: path.file_name().unwrap().to_str().unwrap().to_string(),
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

    fn read_jxl(file_path: String) -> ImageReader {
        let sample = std::fs::read(file_path.clone()).unwrap();
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let (metadata, pixels) = decoder.decode(&sample).unwrap();
        // file_path: .../<test_set>/<orig_image_name>-<distance>-<effort>.jxl
        let path = Path::new(&file_path);
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let extension = path.extension().unwrap().to_str().unwrap();
        let file_name_parts: Vec<&str> = file_name.split("-").collect();
        let orig_image_name = file_name_parts[0..file_name_parts.len() - 2].join("-");

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

        if extension != "jxl" {
            panic!("Not a .jxl file");
        }

        ImageReader {
            image: None,
            file_data: ImageFileData {
                image_name: file_name.clone(),
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

    fn get_file_size(file_path: &String) -> usize {
        let metadata = std::fs::metadata(file_path).unwrap();
        let size = metadata.len();
        size as usize
    }

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

    pub fn calculate_mse(orig_image_path: &String, comp_image_path: &String) -> f64 {
        let orig_image = image::open(orig_image_path).unwrap();
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let comp_image = std::fs::read(comp_image_path.clone()).unwrap();
        let (comp_metadata, comp_pixels) = decoder.decode(&comp_image).unwrap();
        //        let orig_image = match ColorType::get_jxl_color_space(&comp_metadata, &comp_pixels) {
        //            ColorType::L8 => orig_image.to_luma8(),
        //            ColorType::La8 => orig_image.to_luma_alpha8(),
        //            ColorType::Rgb8 => orig_image.to_rgb8(),
        //            ColorType::Rgba8 => orig_image.to_rgba8(),
        //            ColorType::L16 => orig_image.to_luma16(),
        //            ColorType::La16 => orig_image.to_luma_alpha16(),
        //            ColorType::Rgb16 => orig_image.to_rgb16(),
        //            ColorType::Rgba16 => orig_image.to_rgba16(),
        //            ColorType::Rgb32F => orig_image.to_rgb32f(),
        //            ColorType::Rgba32F => orig_image.to_rgba32f(),
        //        };
        let orig_image = match ColorType::get_jxl_color_space(&comp_metadata, &comp_pixels) {
            ColorType::Rgb8 => orig_image.to_rgb8(),
            _ => todo!(),
        };
        let orig_image = orig_image.as_flat_samples();
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

    pub fn calculate_psnr(mse: f64, max_value: f64) -> f64 {
        10.0 * ((max_value * max_value) / mse).log10()
    }
}
