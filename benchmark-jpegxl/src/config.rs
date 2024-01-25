pub struct Config {
    pub application_name: &'static str,
    pub application_author: &'static str,
    pub application_version: &'static str,
    pub application_description: &'static str,

    pub benchmark_dir_path: &'static str,
    pub docker_file_path: &'static str,
    pub local_test_image_dir_path: &'static str,
    pub docker_test_image_dir_path: &'static str,

    pub use_temp_dir: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            application_name: "JPEG-XL Benchmark",
            application_author: "",
            application_version: "0.1.0",
            application_description: "A benchmark tool for the JPEG-XL image format",

            benchmark_dir_path: "./benchmarks",
            docker_file_path: "./Dockerfile",
            local_test_image_dir_path: "./test_images",
            docker_test_image_dir_path: "/test_images",

            use_temp_dir: false,
        }
    }
}
