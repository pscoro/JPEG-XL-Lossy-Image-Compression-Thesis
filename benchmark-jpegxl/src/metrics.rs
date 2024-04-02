use crate::{docker_manager::DockerManager, image_reader::ImageReader};

use std::io::BufRead;
use std::process::Command;

/// Calculate the ratio of the file sizes of the original and compressed files.
///
/// # Arguments
/// * `orig` - The original file size.
/// * `comp` - The compressed file size.
/// * `denom` - The denominator to use for the ratio. Either "orig" or "comp".
///
/// # Returns
/// The ratio of the file sizes. orig / comp if denom is "comp", comp / orig if denom is "orig".
pub fn file_size_ratio(orig: usize, comp: usize, denom: &str) -> f64 {
    // Check if denominator is 0.
    if (orig == 0 && denom == "orig") || (comp == 0 && denom == "comp") {
        return 0.0;
    }

    match denom {
        "orig" => comp as f64 / orig as f64,
        "comp" => orig as f64 / comp as f64,
        _ => panic!("Invalid denominator for file size ratio"),
    }
}

/// Calculate the mean squared error (MSE) between two images.
/// Just a wrapper around the ImageReader method for a more consistent API.
///
/// # Arguments
/// * `orig_image_path` - The path to the original image.
/// * `comp_image_path` - The path to the compressed image.
///
/// # Returns
/// The mean squared error between the two images.
pub fn calculate_mse(orig_image_path: &String, comp_image_path: &String) -> f64 {
    ImageReader::calculate_mse(orig_image_path, comp_image_path)
}

/// Calculate the peak signal-to-noise ratio (PSNR) between two images.
/// Just a wrapper around the ImageReader method for a more consistent API.
///
/// # Arguments
/// * `orig_image_path` - The path to the original image.
/// * `comp_image_path` - The path to the compressed image.
///
/// # Returns
/// The peak signal-to-noise ratio between the two images.
pub fn calculate_psnr(orig_image_path: &String, comp_image_path: &String, max_value: f64) -> f64 {
    let mse = calculate_mse(orig_image_path, comp_image_path);
    ImageReader::calculate_psnr(mse, max_value)
}

/// Calculate the structural similarity index (SSIM) between two images.
/// Uses the ImageMagick `compare` command locally with the SSIM metric.
///
/// # Arguments
/// * `orig_image_path` - The path to the original image.
/// * `comp_image_path` - The path to the compressed image.
///
/// # Returns
/// The structural similarity index between the two images.
pub fn calculate_ssim(orig_image_path: &String, comp_image_path: &String) -> f64 {
    // $ magick compare -metric SSIM orig.png comp.png diff.png
    let result = Command::new("magick")
        .arg("compare")
        .arg("-metric")
        .arg("SSIM")
        .arg(orig_image_path)
        .arg(comp_image_path)
        .arg("null:")
        .output();

    result
        .unwrap()
        .stderr
        .lines()
        .next()
        .unwrap()
        .expect("Error calculating SSIM")
        .parse::<f64>()
        .unwrap()
}

/// Calculate the Butteraugli perceptual distance between two images.
/// Uses the libjxl Butteraugli tool in a Docker container to calculate the distance.
///
/// # Arguments
/// * `docker_input_path` - The path to the original image in the Docker container.
/// * `docker_output_path` - The path to the compressed image in the Docker container.
/// * `docker_manager` - The DockerManager instance to execute the Butteraugli command.
///
/// # Returns
/// The Butteraugli perceptual distance between the two images and the p-norm value, as a tuple.
pub fn calculate_butteraugli(
    docker_input_path: &str,
    docker_output_path: &str,
    docker_manager: &DockerManager,
) -> (f64, f64) {
    let result = docker_manager.execute_butteraugli(
        docker_input_path.to_string().clone(),
        docker_output_path.to_string().clone(),
    );
    let result = result.unwrap();
    let output = result.clone().unwrap_err();

    // Butteraugli may fail because of libpng warning: iCCP: known incorrect sRGB profile
    let butteraugli = output.lines().next().unwrap().parse::<f64>().unwrap_or(0.0);

    let pnorm = output
        .lines()
        .last()
        .unwrap()
        .split_whitespace()
        .last()
        .unwrap();
    let pnorm = pnorm.parse::<f64>().unwrap_or(0.0);

    (butteraugli, pnorm)
}

/// Calculate the SSIMULACRA2 perceptual distance between two images.
/// Uses the libjxl SSIMULACRA2 tool in a Docker container to calculate the distance.
///
/// # Arguments
/// * `docker_input_path` - The path to the original image in the Docker container.
/// * `docker_output_path` - The path to the compressed image in the Docker container.
/// * `docker_manager` - The DockerManager instance to execute the SSIMULACRA2 command.
///
/// # Returns
/// The SSIMULACRA2 perceptual distance between the two images.
pub fn calculate_ssimulacra2(
    docker_input_path: &str,
    docker_output_path: &str,
    docker_manager: &DockerManager,
) -> f64 {
    let result = docker_manager.execute_ssimulacra2(
        docker_input_path.to_string().clone(),
        docker_output_path.to_string().clone(),
    );
    let output = result.unwrap().unwrap();
    output.lines().next().unwrap().parse::<f64>().unwrap()
}
