pub struct Config {
    pub APPLICATION_NAME: &'static str,
    pub APPLICATION_AUTHOR: &'static str,
    pub APPLICATION_VERSION: &'static str,
    pub APPLICATION_DESCRIPTION: &'static str,

    pub BENCHMARK_DIR_PATH: &'static str,
    pub DOCKER_FILE_PATH: &'static str,
    pub LOCAL_TEST_IMAGE_DIR_PATH: &'static str,
    pub DOCKER_TEST_IMAGE_DIR_PATH: &'static str,

    pub USE_TEMP_DIR: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            APPLICATION_NAME: "JPEG-XL Benchmark Tool",
            APPLICATION_AUTHOR: "",
            APPLICATION_VERSION: "0.1.0",
            APPLICATION_DESCRIPTION: "A benchmark tool for the JPEG-XL image codec",
            BENCHMARK_DIR_PATH: "./benchmarks",
            DOCKER_FILE_PATH: "./Dockerfile",
            LOCAL_TEST_IMAGE_DIR_PATH: "test_images",
            DOCKER_TEST_IMAGE_DIR_PATH: "/test_images",
            USE_TEMP_DIR: false,
        }
    }
}
