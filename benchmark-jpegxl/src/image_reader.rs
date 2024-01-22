use serde::{Serialize, Serializer};
use serde_derive::Serialize;

use image::DynamicImage;
use std::path::Path;

use jpegxl_rs::decode::{JxlDecoder, Metadata, Pixels};
use jpegxl_rs::decoder_builder;

#[derive(Debug)]
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
            }
            2 => match pixels {
                Pixels::Uint8(_) => ColorType::La8,
                Pixels::Uint16(_) => ColorType::La16,
                _ => panic!("Unknown jxl color space"),
            }
            3 => match pixels {
                Pixels::Uint8(_) => ColorType::Rgb8,
                Pixels::Uint16(_) => ColorType::Rgb16,
                _ => panic!("Unknown jxl color space"),
            }
            4 => match pixels {
                Pixels::Uint8(_) => ColorType::Rgba8,
                Pixels::Uint16(_) => ColorType::Rgba16,
                _ => panic!("Unknown jxl color space"),
            }
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

#[derive(Debug, Serialize)]
pub struct ImageFileData {
    pub file_path: String,
    pub width: u32,
    pub height: u32,
    pub file_size: usize,
    pub raw_size: usize,
    pub file_format: ImageFormat,
    pub color_space: ColorType,
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
        if extension == "jxl" {
            return ImageReader::read_jxl(file_path);
        }
        let image = image::open(&file_path).unwrap();

        ImageReader {
            image: Some(image.clone()),
            file_data: ImageFileData {
                file_path: file_path.clone(),
                width: image.width(),
                height: image.height(),
                file_size: ImageReader::get_file_size(&file_path),
                raw_size: ImageReader::get_raw_size(&file_path),
                file_format: ImageReader::get_format(&file_path),
                color_space: image.color().into(),
            },
        }
    }

    fn read_jxl(file_path: String) -> ImageReader {
        let sample = std::fs::read(file_path.clone()).unwrap();
        let decoder: JxlDecoder = decoder_builder().build().unwrap();
        let (metadata, pixels) = decoder.decode(&sample).unwrap();
        ImageReader {
            image: None,
            file_data: ImageFileData {
                file_path: file_path.clone(),
                width: metadata.width.clone(),
                height: metadata.height.clone(),
                file_size: ImageReader::get_file_size(&file_path),
                raw_size: ImageReader::get_raw_size(&file_path),
                file_format: ImageReader::get_format(&file_path),
                color_space: ColorType::get_jxl_color_space(&metadata, &pixels),
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
            _ => todo!(),
        };
        let size = width * height * bytes_per_pixel;
        size as usize
    }
}
