pub struct Config {
    pub benchmark_dir_path: &'static str,
    pub docker_file_path: &'static str,
    pub local_test_image_dir_path: &'static str,
    pub docker_test_image_dir_path: &'static str,
    pub num_workers: usize,

    pub use_temp_dir: bool,
    pub libjxl_commit: Option<String>,
    pub compare_to_local: bool,
    pub compare_to_commit: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            benchmark_dir_path: "./benchmarks",
            docker_file_path: "./Dockerfile",
            local_test_image_dir_path: "./test_images",
            docker_test_image_dir_path: "/test_images",
            num_workers: 16,

            use_temp_dir: false,
            libjxl_commit: None,
            compare_to_local: false,
            compare_to_commit: None,
        }
    }
}
