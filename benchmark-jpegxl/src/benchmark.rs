use crate::config::Config;
use crate::context::{Context, DEFAULT_LIBJXL_COMMIT};
use crate::csv_writer::*;
use crate::docker_manager::DockerManager;
use crate::image_reader::{ImageFileData, ImageFormat, ImageReader};
use crate::metrics::*;
use crate::utils::*;

use std::fs;
use std::path::PathBuf;

/// All benchmarks should implement this trait.
/// The run method should be called to run the benchmark.
pub trait Benchmark: Sync + Send {
    fn run(docker_manager: DockerManager, payload: &WorkerPayload);
}

/// Benchmark for JPEG XL compression.
/// Implements the Benchmark trait.
pub struct JXLCompressionBenchmark {}

/// Runs benchmarks on multiple workers.
#[derive(Debug)]
pub struct Benchmarker {
    pub context: Context,
    pub workers: Vec<BenchmarkWorker>,
    pub current_worker_id: usize,
}

/// Represents a worker that runs a benchmark.
/// Contains a DockerManager and a thread handle.
/// The worker may execute commands on the DockerManager.
#[derive(Debug)]
pub struct BenchmarkWorker {
    pub id: usize,
    pub docker_manager: Option<DockerManager>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
    pub payload: Option<WorkerPayload>,
    pub working: bool,
}

/// Represents the payload for a worker.
#[derive(Debug, Clone)]
pub struct WorkerPayload {
    pub context: Context,
    pub current_worker_id: usize,
    pub current_out_orig_path: String,
    pub current_out_comp_path: String,
    pub current_res_orig_path: String,
    pub current_res_comp_path: String,
    pub current_image_name: String,      // "kodim06"
    pub current_image_file_path: String, // "kodim/kodim06.png"
    pub current_image_format: ImageFormat,
    pub current_test_set: String,
}

impl BenchmarkWorker {
    /// Creates a new BenchmarkWorker with the given id and payload.
    ///
    /// # Arguments
    /// * `id` - The id of the worker.
    /// * `payload` - The payload for the worker.
    ///
    /// # Returns
    /// A new BenchmarkWorker.
    pub fn new(id: usize, payload: &WorkerPayload) -> BenchmarkWorker {
        BenchmarkWorker {
            id,
            docker_manager: None,
            thread_handle: None,
            payload: Some(payload.clone()),
            working: false,
        }
    }

    /// Runs the benchmark on the worker.
    /// The benchmark is run on a separate thread owned by the worker.
    ///
    /// # Arguments
    /// * `T` - The benchmark to run.
    pub fn run<T: Benchmark + 'static>(&mut self) {
        let payload = self.payload.as_ref().unwrap().clone();

        // Take and join the thread if for some reason it is still working.
        if self.thread_handle.is_some() {
            let th = self.thread_handle.take();
            match th {
                Some(thread_handle) => {
                    let _ = thread_handle.join();
                    self.thread_handle = None;
                    self.working = false;
                }
                None => {}
            }
        }

        let docker = self.docker_manager.as_mut().unwrap().clone();
        self.working = true;

        // Spawn a new thread to run the benchmark with the given payload.
        self.thread_handle = Some(std::thread::spawn(move || {
            T::run(docker, &payload);
        }));
    }
}

impl WorkerPayload {
    /// Creates or gets the output directory for the current run.
    pub fn get_output_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/output", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets the result directory for the current run.
    pub fn get_result_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/results", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets a directory under the output directory for the current run.
    pub fn get_output_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/output/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets a directory under the result directory for the current run.
    pub fn get_result_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/results/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }
}

impl Benchmarker {
    /// Creates a new Benchmarker with the given config.
    ///
    /// # Arguments
    /// * `config` - The config for the benchmarker.
    ///
    /// # Returns
    /// A new Benchmarker.
    pub fn new(config: &Config) -> Benchmarker {
        // Create the context for the benchmarker out of the config.
        let c = Context {
            benchmark_dir: exists_or_create_dir(&config.benchmark_dir_path).unwrap(),
            test_sets: Benchmarker::get_all_test_set_names(
                config.local_test_image_dir_path.clone(),
            ),
            current_run: Benchmarker::get_current_run(config.benchmark_dir_path.clone()),
            local_test_image_dir: config.local_test_image_dir_path.clone(),
            docker_test_image_dir: config.docker_test_image_dir_path.clone(),
            num_workers: config.num_workers,
            use_temp_dir: config.use_temp_dir,
            libjxl_commit: config.libjxl_commit.clone(),
            compare_to_local: config.compare_to_local,
            compare_to_commit: config.compare_to_commit.clone(),
        };

        // Create a new Benchmarker with the given context.
        let mut b = Benchmarker {
            context: c,
            workers: Vec::new(),
            current_worker_id: 0,
        };

        // Create workers for the benchmarker.
        let config = Config::default();
        for x in 0..b.context.num_workers {
            // Initialize an empty payload for each worker.
            let payload = WorkerPayload {
                context: b.context.clone(),
                current_worker_id: x,
                current_out_orig_path: "".to_string(),
                current_out_comp_path: "".to_string(),
                current_res_orig_path: "".to_string(),
                current_res_comp_path: "".to_string(),
                current_image_name: "".to_string(),
                current_image_file_path: "".to_string(),
                current_image_format: ImageFormat::Unsupported,
                current_test_set: "".to_string(),
            };

            // Create a new worker with the given worker index as id and payload.
            let mut worker = BenchmarkWorker::new(x, &payload);

            // Create and setup a new DockerManager for the worker.
            let mut docker_manager = DockerManager::new(&config.docker_file_path, x);
            let _ = docker_manager.setup(worker.id).unwrap();
            worker.docker_manager = Some(docker_manager);

            // Add the worker to the benchmarker.
            b.workers.push(worker);
        }
        b
    }

    /// Gets the next worker id.
    ///
    /// # Returns
    /// The next worker id to use and increments the current worker id.
    fn get_next_worker_id(&mut self) -> usize {
        let id = self.current_worker_id;
        self.current_worker_id += 1;
        if self.current_worker_id >= self.context.num_workers {
            self.current_worker_id = 0;
        }
        id
    }

    /// Creates or gets the output directory for the current run.
    ///
    /// # Arguments
    /// * `benchmark_dir` - The benchmark directory.
    /// * `current_run` - The current run number.
    ///
    /// # Returns
    /// The path to the output directory.
    pub fn get_output_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/output", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets the result directory for the current run.
    ///
    /// # Arguments
    /// * `benchmark_dir` - The benchmark directory.
    /// * `current_run` - The current run number.
    ///
    /// # Returns
    /// The path to the result directory.
    pub fn get_result_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/results", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets a directory under the output directory for the current run.
    ///
    /// # Arguments
    /// * `file_path` - The file path to create or get a directory for.
    ///
    /// # Returns
    /// The path to the directory.
    pub fn get_output_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/output/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    /// Creates or gets a directory under the result directory for the current run.
    ///
    /// # Arguments
    /// * `file_path` - The file path to create or get a directory for.
    ///
    /// # Returns
    /// The path to the directory.
    pub fn get_result_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/results/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    /// Gets the integer representing the current run of the benchmarker.
    /// The current run number is based on the number-named run directories in the benchmark 
    /// directory. The current run number is the highest number found in the directory plus one.
    ///
    /// # Arguments
    /// * `benchmark_dir` - The benchmark directory.
    ///
    /// # Returns
    /// The current run number.
    pub fn get_current_run(benchmark_dir: String) -> usize {
        let mut current_run = 0;
        let path = PathBuf::from(benchmark_dir.clone());
        if !path.exists() {
            return current_run;
        }

        // Get the highest number-named directory in the benchmark directory.
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            // if file name is not a number, skip it, for temp/ directory in non-temp mode
            if file_name.parse::<usize>().is_err() {
                continue;
            }
            let run = file_name.parse::<usize>().unwrap();
            if run >= current_run {
                current_run = run + 1;
            }
        }
        current_run
    }

    /// Gets all the test set names in the local test image directory.
    /// The test set names are the names of the directories in the local test image directory.
    ///
    /// # Arguments
    /// * `local_test_image_dir` - The local test image directory.
    ///
    /// # Returns
    /// A vector of test set names.
    pub fn get_all_test_set_names(local_test_image_dir: String) -> Vec<String> {
        let mut test_sets = Vec::new();
        let path = PathBuf::from(local_test_image_dir.clone());
        if !path.exists() {
            return test_sets;
        }

        // Add all the directories in the local test image directory to the test sets vector.
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let test_set_dir_name = path.file_name().unwrap().to_str().unwrap();
            // if test_set_dir_name starts with a "." or is not a directory, skip it
            if test_set_dir_name.starts_with(".") || !path.is_dir() {
                continue;
            }
            test_sets.push(test_set_dir_name.to_string());
        }
        test_sets
    }

    /// Waits for the next worker to be available to run a benchmark and returns a mutable
    /// reference to the worker.
    ///
    /// # Returns
    /// A mutable reference to the next available worker.
    pub fn wait_for_available_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.get_next_worker_id();
        let worker = &mut self.workers[id] as &mut BenchmarkWorker;

        // If the worker is working, wait for it to finish.
        if worker.working {
            if let Some(thread_handle) = worker.thread_handle.take() {
                let _ = thread_handle.join();
                worker.thread_handle = None;
                worker.working = false;
            } else {
                panic!("Working worker thread handle is None");
            }
        }
        worker
    }

    /// Gets the current worker and returns a mutable reference to it.
    ///
    /// # Returns
    /// A mutable reference to the current worker.
    pub fn get_current_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.current_worker_id;
        &mut self.workers[id] as &mut BenchmarkWorker
    }

    /// Waits on the current thread for all workers to finish working.
    pub fn wait_for_all_workers(&mut self) {
        
        // Wait for all workers to finish working and join their threads.
        for worker in &mut self.workers {
            if let Some(thread_handle) = worker.thread_handle.take() {
                let _ = thread_handle.join();
                worker.thread_handle = None;
                worker.working = false;
            } else {
                continue;
            }
        }

        // Sanity check to make sure no worker is still flagged as working.
        for worker in &self.workers {
            if worker.working {
                panic!("Worker {} is still working", worker.id);
            }
        }
    }

    /// Runs a benchmark on the benchmarker.
    /// The benchmark is run across all the workers in the benchmarker.
    ///
    /// # Arguments
    /// * `T` - The benchmark to run.
    pub fn run_benchmark<T: Benchmark + 'static>(&mut self) {
        // Set the current run of the context.
        self.context.current_run = Benchmarker::get_current_run(self.context.benchmark_dir.clone());

        // Get the libjxl commit for the benchmark or use the default commit (main).
        let libjxl_commit = self.context.libjxl_commit.clone();
        let mut commit = match libjxl_commit {
            Some(commit) => Some(commit),
            None => Some(DEFAULT_LIBJXL_COMMIT.to_string()),
        };

        // Initialize the benchmark comparison CSVs vector.
        let mut comparison_csvs = Vec::<String>::new();

        // Run the benchmark for each test set.
        let test_sets = self.context.test_sets.clone();
        for test_set in &test_sets {
            // Make sure test_set is a directory.
            let local_test_set_path = dir_exists(format!("{}/{}", self.context.local_test_image_dir, test_set).as_str()).unwrap();

            // Run the benchmark for each commit in the case of a comparison.
            while commit.is_some() {
                // Set up output and result paths.
                let out_orig_path = self.get_output_path_for(
                    format!("orig/{}/{}", test_set, commit.clone().unwrap()).as_str(),
                );
                let out_comp_path = self.get_output_path_for(
                    format!("comp/{}/{}", test_set, commit.clone().unwrap()).as_str(),
                );
                let res_orig_path = self.get_result_path_for(
                    format!("orig/{}/{}", test_set, commit.clone().unwrap()).as_str(),
                );
                let res_comp_path = self.get_result_path_for(
                    format!("comp/{}/{}", test_set, commit.clone().unwrap()).as_str(),
                );

                // Create a context for the worker payload.
                // Set the libjxl commit for the worker payload context.
                let mut context = self.context.clone();
                context.libjxl_commit = Some(commit.clone().unwrap().to_string());

                // Iterate over the images in the local test set path.
                for entry in fs::read_dir(local_test_set_path.clone()).unwrap() {
                    let entry = entry;

                    // Check if entry is a supported image file.
                    // Get extension of the file and check if it is supported.
                    if entry.as_ref().unwrap().path().is_dir() {
                        continue;
                    }
                    match ImageFormat::from_file_name(
                        entry.as_ref().unwrap().path().to_str().unwrap(),
                    ) {
                        ImageFormat::Unsupported => continue,
                        _ => {}
                    }

                    // Wait for the next available worker.
                    let worker = self.wait_for_available_worker();

                    // Clean the libjxl branch on the docker manager of the worker.
                    let _ = worker
                        .docker_manager
                        .as_ref()
                        .unwrap()
                        .clean_libjxl()
                        .unwrap();

                    // Set the current commit of libjxl on the docker manager.
                    if commit.is_some() && commit.as_ref().unwrap() == "local" {
                        // Apply a diff of the local changes to libjxl on the worker container.
                        let _ = worker
                            .docker_manager
                            .as_ref()
                            .unwrap()
                            .apply_local_as_diff()
                            .unwrap();
                    } else {
                        // Set the current commit of libjxl on the worker container.
                        let _ = worker
                            .docker_manager
                            .as_ref()
                            .unwrap()
                            .change_libjxl_commit(commit.clone().unwrap().as_str())
                            .unwrap();
                    }

                    // Re-build libjxl on the docker manager of the worker.
                    let _ = worker
                        .docker_manager
                        .as_ref()
                        .unwrap()
                        .build_libjxl()
                        .unwrap();

                    // Set current image file path and name for the worker payload.
                    let entry = entry.unwrap();
                    let path = entry.path();
                    worker.payload.as_mut().unwrap().current_image_file_path =
                        path.to_str().unwrap().to_string();
                    worker.payload.as_mut().unwrap().current_image_name =
                        path.file_name().unwrap().to_str().unwrap().to_string();

                    // Remove the file extension from the current image name.
                    worker.payload.as_mut().unwrap().current_image_name = worker
                        .payload
                        .as_mut()
                        .unwrap()
                        .current_image_name
                        .as_str()
                        .split(".")
                        .collect::<Vec<&str>>()[0]
                        .to_string();

                    // Set the current output and result directories for the worker payload.
                    worker.payload.as_mut().unwrap().current_out_orig_path = out_orig_path.clone();
                    worker.payload.as_mut().unwrap().current_out_comp_path = out_comp_path.clone();
                    worker.payload.as_mut().unwrap().current_res_orig_path = res_orig_path.clone();
                    worker.payload.as_mut().unwrap().current_res_comp_path = res_comp_path.clone();

                    // Set the current image format and test set for the worker payload.
                    worker.payload.as_mut().unwrap().current_image_format =
                        ImageFormat::from_file_name(
                            &worker.payload.as_ref().unwrap().current_image_file_path,
                        );
                    worker.payload.as_mut().unwrap().current_test_set = test_set.clone();

                    // Set the context for the worker payload.
                    worker.payload.as_mut().unwrap().context = context.clone();

                    // Run the benchmark for the current image on the worker.
                    worker.run::<T>();
                }

                // Add the benchmark result file to the comparison CSVs vector.
                let result_file = format!("{}/comparisons.csv", res_comp_path);
                comparison_csvs.push(result_file.clone());

                // If the benchmark is not a comparison, break here.
                if !self.context.compare_to_local && self.context.compare_to_commit.is_none() {
                    break;
                }

                // Otherwise, set the commit to compare to.
                let compare_to_commit = self.context.clone().compare_to_commit.clone();
                if compare_to_commit.is_some() {
                    commit = Some(compare_to_commit.unwrap().clone());
                    self.context.compare_to_commit = None;
                } else {
                    commit = match &self.context.compare_to_local {
                        true => {
                            self.context.compare_to_local = false;
                            Some("local".to_string())
                        }
                        false => None,
                    };
                }
            }

            // When all workers are finished, both commits have been benchmarked on all images.
            self.wait_for_all_workers();

            // Compare the results of the benchmarks if applicable.
            if comparison_csvs.len() == 2 {
                // TODO: This isn't generalic to all benchmarks, but this doesn't matter if we only have one JPEG XL benchmark at this moment.
                JXLCompressionBenchmark::compare_results(&comparison_csvs[0], &comparison_csvs[1]);
            } else if comparison_csvs.len() == 1 {
                continue;
            } else if comparison_csvs.len() > 2 {
                panic!("More than 2 comparison CSVs found");
            } else {
                panic!("No comparison CSVs found");
            }
        }
    }

    /// Teardown the benchmarker.
    /// Tears down all the docker managers of the workers.
    pub fn teardown(&mut self) {
        for worker in &mut self.workers {
            worker.docker_manager.as_ref().unwrap().teardown().unwrap();
        }
    }
}

impl Benchmark for JXLCompressionBenchmark {
    /// Runs the JPEG XL compression benchmark.
    /// The benchmark will: 
    ///   - compress images with the JPEG XL codec using the cjxl encoder tool.
    ///   - test different distance and effort combinations.
    ///   - compare the original and compressed images.
    ///   - write the results to CSV files.
    ///   - compare the results to the original images.
    ///
    /// # Arguments
    /// * `docker_manager` - The DockerManager to use for running the benchmark, from the worker.
    /// * `payload` - The payload for the benchmark, from the worker.
    fn run(docker_manager: DockerManager, payload: &WorkerPayload) {
        // Get the libjxl commit for the benchmark or use the default commit (main).
        let commit = match &payload.context.libjxl_commit {
            Some(commit) => Some(commit.as_str()),
            None => Some(DEFAULT_LIBJXL_COMMIT),
        };

        // Get the current test set for the benchmark.
        let _test_set = payload.current_test_set.clone();

        // Set up output and result paths.
        let _out_orig_path = payload.current_out_orig_path.clone(); // Not used.
        let out_comp_path = payload.current_out_comp_path.clone();
        let res_orig_path = payload.current_res_orig_path.clone();
        let res_comp_path = payload.current_res_comp_path.clone();

        // Check if the current image is a supported image file.
        match ImageFormat::from_file_name(&payload.current_image_file_path) {
            ImageFormat::Unsupported => panic!("Unsupported image format, this should not happen"),
            _ => {}
        }

        // Get the file path for the current image.
        let file_path = format!(
            "{}/{}",
            payload.context.docker_test_image_dir,
            payload
                .current_image_file_path
                .split(&format!("{}/", &payload.context.local_test_image_dir))
                .collect::<Vec<&str>>()[1]
        );

        // Initialize an ImageReader to read the current image.
        let image_reader = ImageReader::new(
            payload.current_image_file_path.clone().to_string(),
            commit.unwrap().to_string(),
        );

        // Write the original image file data to a CSV file.
        let image_file_data = image_reader.file_data;
        let result_file = format!("{}/results.csv", res_orig_path,);
        let csv_writer = ImageFileDataCSV::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();

        // The JXL compression benchmark tests combinations of the following distances and efforts.
        // TODO: Make these configurable.
        let distances = vec![0.5, 1.0, 1.5, 3.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0];
        let efforts = (5..=9).collect::<Vec<u32>>();

        // Run the compression benchmark for each distance and effort combination.
        for distance in distances {
            for effort in efforts.clone() {
                // Create the compressed image name.
                let comp_image_name = format!(
                    "{}-{}-{}.{}",
                    payload.current_image_name,
                    distance,
                    effort,
                    ImageFormat::JpegXl.to_string()
                );

                // Execute the cjxl encoder on the current image with the current distance and
                // effort on the provided docker manager.
                let _ = docker_manager
                    .execute_cjxl(
                        file_path.to_string().clone(),
                        comp_image_name.clone(),
                        distance,
                        effort,
                    )
                    .unwrap().unwrap();

                // Retrieve the compressed image from the docker manager.
                let src_path = format!("/temp/{}", comp_image_name);
                let dest_path = format!("{}/{}", out_comp_path, comp_image_name);
                docker_manager
                    .retrieve_file(src_path.clone(), dest_path)
                    .unwrap();

                // Read the compressed image file data.
                let image_reader = ImageReader::new(
                    format!("{}/{}", out_comp_path, comp_image_name),
                    commit.unwrap().to_string(),
                );

                // Write the compressed image file data to a CSV file.
                let image_file_data = image_reader.file_data;
                let result_file = format!("{}/results.csv", res_comp_path);
                let csv_writer = ImageFileDataCSV::new();
                csv_writer.write_csv_header(&result_file).unwrap();
                csv_writer
                    .write_csv(&vec![image_file_data.clone()], &result_file)
                    .unwrap();

                // Compare the original and compressed images.
                JXLCompressionBenchmark::compare_to_orig(
                    &image_file_data,
                    &out_comp_path,
                    &res_orig_path,
                    &res_comp_path,
                    &docker_manager,
                    &file_path,
                    &src_path,
                );
            }
        }
    }
}

impl JXLCompressionBenchmark {
    /// Compares JPEG XL benchmarking results from two different commits/versions of the codec.
    /// The comparison results are written to a CSV file.
    ///
    /// # Arguments
    /// * `results_1` - The path to the first run's results CSV file.
    /// * `results_2` - The path to the second run's results CSV file.
    fn compare_results(results_1: &str, results_2: &str) {
        // Initialize a csv handler for reading the comparison results.
        let csv_reader = ComparisonResultCSV::new();

        // Read the comparison results from the CSV files.
        let comparison_results_1 = csv_reader.read_csv(results_1).unwrap();
        let comparison_results_2 = csv_reader.read_csv(results_2).unwrap();

        // Sort the comparison results by image name, in case they are not already in the same
        // order, so that they can be compared.
        let mut comparison_results_1 = comparison_results_1.clone();
        comparison_results_1.sort_by(|a, b| a.orig_image_name.cmp(&b.orig_image_name));
        let mut comparison_results_2 = comparison_results_2.clone();
        comparison_results_2.sort_by(|a, b| a.orig_image_name.cmp(&b.orig_image_name));

        let mut results = Vec::<ComparisonResultDiff>::new();

        // Compare each entry in the results CSVs.
        assert!(comparison_results_1.len() == comparison_results_2.len());
        for i in 0..comparison_results_1.len() {
            // Assert that the right entries are being compared.
            assert!(
                comparison_results_1[i].orig_image_name == comparison_results_2[i].orig_image_name
            );
            assert!(
                comparison_results_1[i].comp_image_name == comparison_results_2[i].comp_image_name
            );
            assert!(comparison_results_1[i].distance == comparison_results_2[i].distance);
            assert!(comparison_results_1[i].effort == comparison_results_2[i].effort);

            // Calculate the differences between the comparison results as:
            //  diff = result_2 - result_1
            let diff_orig_file_size = comparison_results_2[i].orig_file_size as f64
                - comparison_results_1[i].orig_file_size as f64;
            let diff_comp_file_size = comparison_results_2[i].comp_file_size as f64
                - comparison_results_1[i].comp_file_size as f64;
            let diff_orig_raw_size = comparison_results_2[i].orig_raw_size as f64
                - comparison_results_1[i].orig_raw_size as f64;
            let diff_comp_raw_size = comparison_results_2[i].comp_raw_size as f64
                - comparison_results_1[i].comp_raw_size as f64;
            let diff_comp_file_size_ratio = comparison_results_2[i].comp_file_size_ratio
                - comparison_results_1[i].comp_file_size_ratio;
            let diff_raw_file_size_ratio = comparison_results_2[i].raw_file_size_ratio
                - comparison_results_1[i].raw_file_size_ratio;
            let diff_mse = comparison_results_2[i].mse - comparison_results_1[i].mse;
            let diff_psnr = comparison_results_2[i].psnr - comparison_results_1[i].psnr;
            let diff_ssim = comparison_results_2[i].ssim - comparison_results_1[i].ssim;
            let diff_ms_ssim = comparison_results_2[i].ms_ssim - comparison_results_1[i].ms_ssim;
            let diff_butteraugli =
                comparison_results_2[i].butteraugli - comparison_results_1[i].butteraugli;
            let diff_butteraugli_pnorm = comparison_results_2[i].butteraugli_pnorm
                - comparison_results_1[i].butteraugli_pnorm;
            let diff_ssimulacra2 =
                comparison_results_2[i].ssimulacra2 - comparison_results_1[i].ssimulacra2;

            // Create a comparison result difference struct and add it to the results vector.
            let result = ComparisonResultDiff {
                orig_image_name: comparison_results_1[i].orig_image_name.clone(),
                comp_image_name: comparison_results_1[i].comp_image_name.clone(),
                distance: comparison_results_1[i].distance,
                effort: comparison_results_1[i].effort,
                diff_orig_file_size,
                diff_comp_file_size,
                diff_orig_raw_size,
                diff_comp_raw_size,
                diff_comp_file_size_ratio,
                diff_raw_file_size_ratio,
                diff_mse,
                diff_psnr,
                diff_ssim,
                diff_ms_ssim,
                diff_butteraugli,
                diff_butteraugli_pnorm,
                diff_ssimulacra2,
            };
            results.push(result);
        }

        // Initialize a summary comparison result difference struct as zeros.
        let mut summary = ComparisonResultDiff {
            orig_image_name: "Summary".to_string(),
            comp_image_name: "Summary".to_string(),
            distance: 0.0,
            effort: 0,
            diff_orig_file_size: 0.0,
            diff_comp_file_size: 0.0,
            diff_orig_raw_size: 0.0,
            diff_comp_raw_size: 0.0,
            diff_comp_file_size_ratio: 0.0,
            diff_raw_file_size_ratio: 0.0,
            diff_mse: 0.0,
            diff_psnr: 0.0,
            diff_ssim: 0.0,
            diff_ms_ssim: 0.0,
            diff_butteraugli: 0.0,
            diff_butteraugli_pnorm: 0.0,
            diff_ssimulacra2: 0.0,
        };

        // Calculate the average differences between the comparison results.
        for result in &results {
            summary.diff_orig_file_size += result.diff_orig_file_size;
            summary.diff_comp_file_size += result.diff_comp_file_size;
            summary.diff_orig_raw_size += result.diff_orig_raw_size;
            summary.diff_comp_raw_size += result.diff_comp_raw_size;
            summary.diff_comp_file_size_ratio += result.diff_comp_file_size_ratio;
            summary.diff_raw_file_size_ratio += result.diff_raw_file_size_ratio;
            summary.diff_mse += result.diff_mse;
            summary.diff_psnr += result.diff_psnr;
            summary.diff_ssim += result.diff_ssim;
            summary.diff_ms_ssim += result.diff_ms_ssim;
            summary.diff_butteraugli += result.diff_butteraugli;
            summary.diff_butteraugli_pnorm += result.diff_butteraugli_pnorm;
            summary.diff_ssimulacra2 += result.diff_ssimulacra2;
        }

        summary.diff_orig_file_size /= results.len() as f64;
        summary.diff_comp_file_size /= results.len() as f64;
        summary.diff_orig_raw_size /= results.len() as f64;
        summary.diff_comp_raw_size /= results.len() as f64;
        summary.diff_comp_file_size_ratio /= results.len() as f64;
        summary.diff_raw_file_size_ratio /= results.len() as f64;
        summary.diff_mse /= results.len() as f64;
        summary.diff_psnr /= results.len() as f64;
        summary.diff_ssim /= results.len() as f64;
        summary.diff_ms_ssim /= results.len() as f64;
        summary.diff_butteraugli /= results.len() as f64;
        summary.diff_butteraugli_pnorm /= results.len() as f64;
        summary.diff_ssimulacra2 /= results.len() as f64;

        // Initialize a CSV handler for the comparison result differences.
        let csv_writer = ComparisonResultDiffCSV::new();

        // Write the comparison result differences to a CSV file.
        let result_file = format!(
            "{}/comparison_diffs.csv",
            PathBuf::from(results_1).parent().unwrap().to_str().unwrap()
        );
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer.write_csv(&results, &result_file).unwrap();

        // Write the summary to a CSV file.
        let summary_file = format!(
            "{}/summary.csv",
            PathBuf::from(results_1).parent().unwrap().to_str().unwrap()
        );
        csv_writer.write_csv_header(&summary_file).unwrap();
        csv_writer.write_csv(&vec![summary], &summary_file).unwrap();
    }

    /// Compares the compressed image to the original image and produces a result CSV file.
    /// The comparison is done using the following metrics:
    ///  Compression Rate:
    ///   - Original file size to compressed file size ratio
    ///   - Raw image size to compressed file size ratio
    ///
    ///  Image Quality:
    ///   - Mean Squared Error (MSE)
    ///   - Peak Signal-to-Noise Ratio (PSNR)
    ///   - Structural Similarity Index (SSIM)
    ///   - Multi-Scale Structural Similarity Index (MS-SSIM)
    ///   - Butteraugli (also reports the Butteraugli p-norm)
    ///   - SSIMULACRA2
    ///
    /// # Arguments
    /// * `comp_image_data` - The compressed image file data.
    /// * `out_comp_path` - The output compressed image path. (Not currently used)
    /// * `res_orig_path` - The original image results path.
    /// * `res_comp_path` - The compressed image results path.
    /// * `docker_manager` - The DockerManager to use for running the comparison.
    /// * `docker_input_path` - The input path for the Butteraugli and SSIMULACRA2 comparison.
    /// * `docker_output_path` - The output path for the Butteraugli and SSIMULACRA2 comparison.
    fn compare_to_orig(
        comp_image_data: &ImageFileData,
        _out_comp_path: &str, // not currently used
        res_orig_path: &str,
        res_comp_path: &str,
        docker_manager: &DockerManager,
        docker_input_path: &str,
        docker_output_path: &str,
    ) {
        // Initialize a CSV handler for the orig image file data.
        let csv_writer = ImageFileDataCSV::new();

        // Find the original image file data from the original results CSV file.
        let orig_entry = csv_writer
            .find_entry(
                format!("{}/results.csv", res_orig_path).as_str(),
                0,
                format!("{}.png", &comp_image_data.jxl_orig_image_name).as_str(),
            )
            .unwrap();

        // Comparison calculations
        // Original file size to compressed file size ratio
        let comp_file_size_ratio = file_size_ratio(orig_entry.file_size, comp_image_data.file_size);

        // Raw image size to compressed file size ratio
        let raw_file_size_ratio = file_size_ratio(comp_image_data.raw_size, comp_image_data.file_size);

        // MSE
        let mse = calculate_mse(&orig_entry.file_path, &comp_image_data.file_path);

        // PSNR
        let psnr = calculate_psnr(&orig_entry.file_path, &comp_image_data.file_path, 255.0);

        // SSIM
        let ssim = calculate_ssim(&orig_entry.file_path, &comp_image_data.file_path);

        // MS-SSIM
        // TODO: Implement MS-SSIM.

        // Butteraugli
        let (butteraugli, pnorm) =
            calculate_butteraugli(docker_input_path, docker_output_path, docker_manager);

        // SSIMULACRA2
        let ssimulacra2 =
            calculate_ssimulacra2(docker_input_path, docker_output_path, docker_manager);

        // Create the comparison result struct.
        let comparison_result = ComparisonResult {
            orig_image_name: orig_entry.image_name.clone(),
            comp_image_name: comp_image_data.image_name.clone(),
            distance: comp_image_data.jxl_distance.into(),
            effort: comp_image_data.jxl_effort.into(),
            orig_file_size: orig_entry.file_size as u64,
            comp_file_size: comp_image_data.file_size as u64,
            orig_raw_size: orig_entry.raw_size as u64,
            comp_raw_size: comp_image_data.raw_size as u64,
            comp_file_size_ratio,
            raw_file_size_ratio,
            mse,
            psnr,
            ssim,
            ms_ssim: 0.0,
            butteraugli,
            butteraugli_pnorm: pnorm,
            ssimulacra2,
        };

        // The comparison result is stored in a CSV file under the result comparison directory.
        let result_file = format!("{}/comparisons.csv", res_comp_path,);

        // Initialize a CSV handler for the comparison result.
        let csv_writer = ComparisonResultCSV::new();

        // Write the comparison result to the CSV file.
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![comparison_result], &result_file)
            .unwrap();
    }
}

impl PartialEq for BenchmarkWorker {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BenchmarkWorker {}

impl std::hash::Hash for BenchmarkWorker {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Clone for Benchmarker {
    fn clone(&self) -> Self {
        Benchmarker {
            context: self.context.clone(),
            workers: Vec::new(),
            current_worker_id: self.current_worker_id,
        }
    }
}
