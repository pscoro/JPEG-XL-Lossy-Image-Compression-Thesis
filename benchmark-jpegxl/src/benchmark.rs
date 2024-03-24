use crate::config::Config;
use crate::context::{Context, DEFAULT_LIBJXL_COMMIT};
use crate::csv_writer::*;
use crate::docker_manager::DockerManager;
use crate::image_reader::{ImageFileData, ImageFormat, ImageReader};
use crate::metrics::*;
use crate::utils::*;

use std::fs;
use std::path::PathBuf;

/**
 * Benchmark trait
 * All benchmarks should implement this trait.
 * The run method should be called to run the benchmark.
 */
pub trait Benchmark: Sync + Send {
    fn run(docker_manager: DockerManager, payload: &WorkerPayload);
}

/**
 * JXLCompressionBenchmark
 * Benchmark for JPEG XL compression.
 * Implements the Benchmark trait.
 */
pub struct JXLCompressionBenchmark {}

/**
 * Benchmarker
 * State for running benchmarks.
 */
#[derive(Debug)]
pub struct Benchmarker {
    pub context: Context,
    pub workers: Vec<BenchmarkWorker>,
    pub current_worker_id: usize,
}

#[derive(Debug)]
pub struct BenchmarkWorker {
    pub id: usize,
    pub docker_manager: Option<DockerManager>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
    pub payload: Option<WorkerPayload>,
    pub working: bool,
}

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

impl BenchmarkWorker {
    pub fn new(id: usize, payload: &WorkerPayload) -> BenchmarkWorker {
        BenchmarkWorker {
            id,
            docker_manager: None,
            thread_handle: None,
            payload: Some(payload.clone()),
            working: false,
        }
    }

    pub fn run<T: Benchmark + 'static>(&mut self) {
        let payload = self.payload.as_ref().unwrap().clone();
        if self.thread_handle.is_some() {
            let th = self.thread_handle.take();
            match th {
                Some(thread_handle) => {
                    thread_handle.join();
                    self.thread_handle = None;
                    self.working = false;
                }
                None => {}
            }
        }
        let docker = self.docker_manager.as_mut().unwrap().clone();
        self.working = true;
        self.thread_handle = Some(std::thread::spawn(move || {
            T::run(docker, &payload);
        }));
    }
}

impl WorkerPayload {
    pub fn get_output_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/output", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_result_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/results", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_output_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/output/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_result_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/results/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
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

impl Benchmarker {
    pub fn new(config: &Config) -> Benchmarker {
        let dir = PathBuf::from(config.benchmark_dir_path.clone());
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
        }

        let c = Context {
            benchmark_dir: config.benchmark_dir_path.clone(),
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

        let mut b = Benchmarker {
            context: c,
            workers: Vec::new(),
            current_worker_id: 0,
        };
        let config = Config::default();
        for x in 0..b.context.num_workers {
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
            let mut worker = BenchmarkWorker::new(x, &payload);
            let mut docker_manager = DockerManager::new(&config.docker_file_path, x);
            let _ = docker_manager.setup(worker.id).unwrap();
            worker.docker_manager = Some(docker_manager);
            b.workers.push(worker);
        }
        b
    }

    fn get_next_worker_id(&mut self) -> usize {
        let id = self.current_worker_id;
        self.current_worker_id += 1;
        if self.current_worker_id >= self.context.num_workers {
            self.current_worker_id = 0;
        }
        id
    }

    pub fn get_output_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/output", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_result_dir(benchmark_dir: &str, current_run: usize) -> String {
        let path = format!("{}/{}/results", benchmark_dir, current_run);
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_output_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/output/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_result_path_for(&self, file_path: &str) -> String {
        let path = format!(
            "{}/{}/results/{}",
            self.context.benchmark_dir, self.context.current_run, file_path
        );
        exists_or_create_dir(&path).unwrap()
    }

    pub fn get_current_run(benchmark_dir: String) -> usize {
        let mut current_run = 0;
        let path = PathBuf::from(benchmark_dir.clone());
        if path.exists() {
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
        }
        current_run
    }

    pub fn get_all_test_set_names(local_test_image_dir: String) -> Vec<String> {
        let mut test_sets = Vec::new();
        let path = PathBuf::from(local_test_image_dir.clone());
        if path.exists() {
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
        }
        test_sets
    }

    pub fn wait_for_available_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.get_next_worker_id();
        let worker = &mut self.workers[id] as &mut BenchmarkWorker;
        if worker.working {
            if let Some(thread_handle) = worker.thread_handle.take() {
                println!("Waiting for worker {} to finish", id);
                thread_handle.join();
                worker.thread_handle = None;
                worker.working = false;
            } else {
                panic!("Working worker thread handle is None");
            }
        }
        worker
    }

    pub fn get_current_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.current_worker_id;
        &mut self.workers[id] as &mut BenchmarkWorker
    }

    pub fn wait_for_all_workers(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread_handle) = worker.thread_handle.take() {
                println!("Waiting for worker {} to finish", worker.id);
                thread_handle.join();
                worker.thread_handle = None;
                worker.working = false;
            } else {
                continue;
            }
        }

        // Sanity check
        for worker in &self.workers {
            if worker.working {
                panic!("Worker {} is still working", worker.id);
            }
        }
    }

    pub fn run_benchmark<T: Benchmark + 'static>(&mut self) {
        // Set the current run of the benchmarker.
        self.context.current_run = Benchmarker::get_current_run(self.context.benchmark_dir.clone());

        // Get the libjxl commit for the benchmark.
        let libjxl_commit = self.context.libjxl_commit.clone();
        let mut commit = match libjxl_commit {
            Some(commit) => Some(commit),
            None => Some(DEFAULT_LIBJXL_COMMIT.to_string()),
        };

        // Initialize the comparison CSVs vector.
        let mut comparison_csvs = Vec::<String>::new();

        // Run the benchmark for each test set.
        let test_sets = self.context.test_sets.clone();
        for test_set in &test_sets {
            // Make sure test_set is a directory.
            let path = PathBuf::from(format!(
                "{}/{}",
                self.context.local_test_image_dir, test_set
            ));
            if !path.is_dir() {
                continue;
            }

            // Get the local test set path for the current test set.
            let local_test_set_path = format!("{}/{}", self.context.local_test_image_dir, test_set);

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
                    println!(
                        "Running benchmark for image: {}",
                        entry.as_ref().unwrap().path().to_str().unwrap()
                    );
                    let entry = entry;

                    // Check if entry is an image file.
                    // Get extension of the file and check if it is supported.
                    if entry.as_ref().unwrap().path().is_dir() {
                        continue;
                    }
                    println!("1");
                    match ImageFormat::from_file_name(
                        entry.as_ref().unwrap().path().to_str().unwrap(),
                    ) {
                        ImageFormat::Unsupported => continue,
                        _ => {}
                    }

                    println!("2");
                    // Get the next worker.
                    let worker = self.wait_for_available_worker();

                    println!("3");
                    // Clean the libjxl branch on the docker manager.
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

                    println!("4");
                    let _ = worker
                        .docker_manager
                        .as_ref()
                        .unwrap()
                        .build_libjxl()
                        .unwrap();

                    println!("5");
                    // Set current image file path and name for the worker payload.
                    let entry = entry.unwrap();
                    let path = entry.path();
                    worker.payload.as_mut().unwrap().current_image_file_path =
                        path.to_str().unwrap().to_string();
                    worker.payload.as_mut().unwrap().current_image_name =
                        path.file_name().unwrap().to_str().unwrap().to_string();

                    println!("6");
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

                    println!("7");
                    // Set the current output and result directories for the worker payload.
                    worker.payload.as_mut().unwrap().current_out_orig_path = out_orig_path.clone();
                    worker.payload.as_mut().unwrap().current_out_comp_path = out_comp_path.clone();
                    worker.payload.as_mut().unwrap().current_res_orig_path = res_orig_path.clone();
                    worker.payload.as_mut().unwrap().current_res_comp_path = res_comp_path.clone();

                    println!("8");
                    // Set the current image format and test set for the worker payload.
                    worker.payload.as_mut().unwrap().current_image_format =
                        ImageFormat::from_file_name(
                            &worker.payload.as_ref().unwrap().current_image_file_path,
                        );
                    worker.payload.as_mut().unwrap().current_test_set = test_set.clone();

                    // Set the context for the worker payload.
                    worker.payload.as_mut().unwrap().context = context.clone();

                    println!("9");
                    // Run the benchmark for the current image on the worker.
                    worker.run::<T>();
                }

                // Add the result file to the comparison CSVs vector.
                let result_file = format!("{}/comparisons.csv", res_comp_path);
                comparison_csvs.push(result_file.clone());

                if !self.context.compare_to_local && self.context.compare_to_commit.is_none() {
                    break;
                }

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

            self.wait_for_all_workers();

            if comparison_csvs.len() == 2 {
                //println!("Comparing results");
                //println!("Comparing {} to {}", comparison_csvs[0], comparison_csvs[1]);
                JXLCompressionBenchmark::compare_results(&comparison_csvs[0], &comparison_csvs[1]);
            } else if comparison_csvs.len() == 1 {
                println!("Only 1 comparison CSV found");
            } else if comparison_csvs.len() > 2 {
                panic!("More than 2 comparison CSVs found");
            } else {
                panic!("No comparison CSVs found");
            }
        }
    }

    pub fn teardown(&mut self) {
        for worker in &mut self.workers {
            worker.docker_manager.as_ref().unwrap().teardown().unwrap();
        }
    }
}

/*pub struct CollectImageMetadataBenchmark {}

impl Benchmark for CollectImageMetadataBenchmark {
    fn run(&self, image_name: &str, file_path: &str, output_path: String, result_path: String) {
        drop(output_path);
        println!(
            "Running Collect Image Metadata Benchmark for image {}",
            image_name
        );
        let image_reader = ImageReader::new(file_path.to_string());
        let image_file_data = image_reader.file_data;
        //        let output_path = format!("{}/{}.csv", output_path, image_name);
        let result_file = format!("{}/{}.csv", result_path, image_name);

        let csv_writer = ImageFileDataCSV::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();
    }
}*/

impl Benchmark for JXLCompressionBenchmark {
    fn run(docker_manager: DockerManager, payload: &WorkerPayload) {
        // Get the libjxl commit for the benchmark.
        let commit = match &payload.context.libjxl_commit {
            Some(commit) => Some(commit.as_str()),
            None => Some(DEFAULT_LIBJXL_COMMIT),
        };

        //println!("Commit: {}", commit.unwrap());

        // Get the current test set for the benchmark.
        let _test_set = payload.current_test_set.clone();

        // Set up output and result paths.
        let _out_orig_path = payload.current_out_orig_path.clone();
        let out_comp_path = payload.current_out_comp_path.clone();
        let res_orig_path = payload.current_res_orig_path.clone();
        let res_comp_path = payload.current_res_comp_path.clone();

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
        let distances = vec![
            /*0.0,*/ 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, /*25.0*/
        ];
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
                //println!(
                //    "Running JXL Compression Benchmark for image {} at path {}",
                //    comp_image_name.clone(), file_path.to_string().clone()
                //);
                println!(
                    "Running JXL Compression Benchmark for image {} at path {} with distance {} and effort {}",
                    comp_image_name.clone(),
                    file_path.to_string().clone(),
                    distance,
                    effort
                );
                let result = docker_manager
                    .execute_cjxl(
                        file_path.to_string().clone(),
                        comp_image_name.clone(),
                        distance,
                        effort,
                    )
                    .unwrap();
                //println!("Result: {:?}", result);

                let _ = result.unwrap();

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
    fn compare_results(results_1: &str, results_2: &str) {
        //println!("Comparing results\n\n");
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

        // print CSVs
        println!("CSV 1: {:?}", comparison_results_1);
        println!("CSV 2: {:?}", comparison_results_2);

        let mut results = Vec::<ComparisonResultDiff>::new();

        // Compare the comparison results.
        assert!(comparison_results_1.len() == comparison_results_2.len());
        for i in 0..comparison_results_1.len() {
            println!(
                "Comparing {} to {}",
                comparison_results_1[i].orig_image_name, comparison_results_2[i].orig_image_name
            );
            assert!(
                comparison_results_1[i].orig_image_name == comparison_results_2[i].orig_image_name
            );
            assert!(
                comparison_results_1[i].comp_image_name == comparison_results_2[i].comp_image_name
            );
            assert!(comparison_results_1[i].distance == comparison_results_2[i].distance);
            assert!(comparison_results_1[i].effort == comparison_results_2[i].effort);

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

        // Write the comparison result differences to a CSV file.
        let result_file = format!(
            "{}/comparison_diffs.csv",
            PathBuf::from(results_1).parent().unwrap().to_str().unwrap()
        );
        let csv_writer = ComparisonResultDiffCSV::new();
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
        //println!("GUGHGH {}/results.csv", res_orig_path);
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
        let raw_file_size_ratio = file_size_ratio(orig_entry.raw_size, comp_image_data.raw_size);

        // MSE
        let mse = calculate_mse(&orig_entry.file_path, &comp_image_data.file_path);

        // PSNR
        let psnr = calculate_psnr(&orig_entry.file_path, &comp_image_data.file_path, 255.0);

        // SSIM
        let ssim = calculate_ssim(&orig_entry.file_path, &comp_image_data.file_path);

        // MS-SSIM
        // TODO: Implement MS-SSIM

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
        //println!("BLARGH {}/comparisons.csv", res_comp_path);
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
