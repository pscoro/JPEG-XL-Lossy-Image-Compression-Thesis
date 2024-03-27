/// Configuration for the benchmarking tool.
pub struct Config {
    pub benchmark_dir_path: String,
    pub docker_file_path: String,
    pub local_test_image_dir_path: String,
    pub docker_test_image_dir_path: String,
    pub num_workers: usize,

    pub use_temp_dir: bool,
    pub libjxl_commit: Option<String>,
    pub compare_to_local: bool,
    pub compare_to_commit: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            benchmark_dir_path: "./benchmarks".to_string(),
            docker_file_path: "./Dockerfile".to_string(),
            local_test_image_dir_path: "./test_images".to_string(),
            docker_test_image_dir_path: "/test_images".to_string(),
            num_workers: 6,

            use_temp_dir: false,
            libjxl_commit: None,
            compare_to_local: false,
            compare_to_commit: None,
        }
    }
}
