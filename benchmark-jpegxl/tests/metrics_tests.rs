use approx::relative_eq;

use benchmark_jpegxl::metrics::*;

const TEST_ORIG_IMAGES: [&str; 5] = [
	"tests/images/orig/1.png",
	"tests/images/orig/2.png",
	"tests/images/orig/3.png",
	"tests/images/orig/4.png",
	"tests/images/orig/5.png",
];

const TEST_COMP_IMAGES: [&str; 5] = [
	"tests/images/comp/1.jxl",
	"tests/images/comp/2.jxl",
	"tests/images/comp/3.jxl",
	"tests/images/comp/4.jxl",
	"tests/images/comp/5.jxl",
];

#[test]
fn test_file_size_ratio() {
	let orig = [1, 2, 3, 4, 5];
	let comp = [2, 4, 6, 8, 10];

	let expected_orig = [
		[2.0, 4.0, 6.0, 8.0, 10.0],
		[1.0, 2.0, 3.0, 4.0, 5.0],
		[0.6666666666666666, 1.3333333333333333, 2.0, 2.6666666666666665, 3.3333333333333335],
		[0.5, 1.0, 1.5, 2.0, 2.5],
		[0.4, 0.8, 1.2, 1.6, 2.0],
	];
	let expected_comp = [
		[0.5, 0.25, 0.16666666666666666, 0.125, 0.1],
		[1.0, 0.5, 0.3333333333333333, 0.25, 0.2],
		[1.5, 0.75, 0.5, 0.375, 0.3],
		[2.0, 1.0, 0.6666666666666666, 0.5, 0.4],
		[2.5, 1.25, 0.8333333333333334, 0.625, 0.5],
	];

	// Test orig / comp and comp / orig
	for i in 0..orig.len() {
		for j in 0..comp.len() {
			let ratio_to_orig = file_size_ratio(orig[i], comp[j], "orig");
			let ratio_to_comp = file_size_ratio(orig[i], comp[j], "comp");

			relative_eq!(ratio_to_orig, expected_orig[i][j], epsilon = f64::EPSILON);
			relative_eq!(ratio_to_comp, expected_comp[i][j], epsilon = f64::EPSILON);
		}
	}

	// Test division by zero
	// TODO: Should this be an error?
	assert_eq!(file_size_ratio(0, 0, "orig"), 0.0);
	assert_eq!(file_size_ratio(0, 0, "comp"), 0.0);
}

#[test]
fn test_calculate_mse() {
	let orig = TEST_ORIG_IMAGES.to_vec();
	let comp = TEST_COMP_IMAGES.to_vec();

	// TODO: Fill in expected values
	let expected = [
        68.3989, 133.789, 4.04149, 17.5837, 282.689,
	];

	for i in 0..orig.len() {
		let mse = calculate_mse(&(orig[i].to_string()), &(comp[i].to_string()));
		relative_eq!(mse, expected[i], epsilon = f64::EPSILON);
	}
}

#[test]
fn test_calculate_psnr() {
	let orig = TEST_ORIG_IMAGES.to_vec();
	let comp = TEST_COMP_IMAGES.to_vec();

    let max_val = 255.0;

	// TODO: Fill in expected values
	let expected = [
        29.8142, 26.9005, 42.0993, 35.7136, 23.6516,
	];

	for i in 0..orig.len() {
		let psnr = calculate_psnr(&(orig[i].to_string()), &(comp[i].to_string()), max_val);
		relative_eq!(psnr, expected[i], epsilon = f64::EPSILON);
	}
}

#[test]
fn test_calculate_ssim() {
	let orig = TEST_ORIG_IMAGES.to_vec();
	let comp = TEST_COMP_IMAGES.to_vec();

	// TODO: Fill in expected values
	let expected = [
        0.783046, 0.861185, 0.993466, 0.96873, 0.632151,
	];

	for i in 0..orig.len() {
		let ssim = calculate_ssim(&(orig[i].to_string()), &(comp[i].to_string()));
		relative_eq!(ssim, expected[i], epsilon = f64::EPSILON);
	}
}
