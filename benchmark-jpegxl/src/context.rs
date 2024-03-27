/// Context struct that holds all the information needed to run the benchmark.
#[derive(Debug, Clone)]
pub struct Context {
    pub benchmark_dir: String,
    pub test_sets: Vec<String>,
    pub current_run: usize,
    pub local_test_image_dir: String,
    pub docker_test_image_dir: String,
    pub num_workers: usize,
    pub use_temp_dir: bool,
    pub libjxl_commit: Option<String>,
    pub compare_to_local: bool,
    pub compare_to_commit: Option<String>,
}

/// Default values for the context struct.
pub const DEFAULT_LIBJXL_COMMIT: &str = "main";
