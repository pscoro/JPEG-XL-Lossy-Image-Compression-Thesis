use crate::{docker_manager::DockerManager, image_reader::ImageReader};

use std::io::BufRead;
use std::process::Command;

pub fn file_size_ratio(orig: usize, comp: usize) -> f64 {
    if comp == 0 {
        return 0.0;
    }
    (orig as f64) / (comp as f64)
}

pub fn calculate_mse(orig_image_path: &String, comp_image_path: &String) -> f64 {
    ImageReader::calculate_mse(orig_image_path, comp_image_path)
}

pub fn calculate_psnr(orig_image_path: &String, comp_image_path: &String, max_value: f64) -> f64 {
    let mse = calculate_mse(orig_image_path, comp_image_path);
    ImageReader::calculate_psnr(mse, max_value)
}

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
    println!("Result: {:?}", result);
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

pub fn calculate_butteraugli(
    docker_input_path: &str,
    docker_output_path: &str,
    docker_manager: &DockerManager,
) -> (f64, f64) {
    let result = docker_manager.execute_butteraugli(
        docker_input_path.to_string().clone(),
        docker_output_path.to_string().clone(),
    );
    let output = result.unwrap().unwrap_err();
    let butteraugli = output.lines().next().unwrap().parse::<f64>().unwrap();
    let pnorm = output
        .lines()
        .last()
        .unwrap()
        .split_whitespace()
        .last()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    (butteraugli, pnorm)
}

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
