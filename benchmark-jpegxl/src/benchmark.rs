use crate::config::Config;
use crate::csv_writer::CSVWriter;
use crate::csv_writer::ImageFileDataCSVWriter;
use crate::docker_manager::DockerManager;
use crate::image_reader::{ImageFormat, ImageReader};

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct BenchmarkWorker {
    pub id: usize,
    pub docker_manager: Option<DockerManager>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
    pub benchmark_dir: String,
    pub test_sets: Vec<String>,
    pub current_run: usize,
    pub local_test_image_dir: String,
    pub docker_test_image_dir: String,
    pub current_worker_id: usize,
    pub current_output_dir: String,
    pub current_result_dir: String,
    pub current_image_name: String, // "kodim06"
    pub current_image_file_path: String, // "kodim/kodim06.png"
    pub current_image_format: ImageFormat,
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

impl Clone for BenchmarkWorker {
    fn clone(&self) -> Self {
        BenchmarkWorker {
            id: self.id,
            docker_manager: None,
            thread_handle: None,
            benchmark_dir: self.benchmark_dir.clone(),
            test_sets: self.test_sets.clone(),
            current_run: self.current_run,
            local_test_image_dir: self.local_test_image_dir.clone(),
            docker_test_image_dir: self.docker_test_image_dir.clone(),
            current_worker_id: self.current_worker_id,
            current_output_dir: self.current_output_dir.clone(),
            current_result_dir: self.current_result_dir.clone(),
            current_image_name: self.current_image_name.clone(),
            current_image_file_path: self.current_image_file_path.clone(),
            current_image_format: self.current_image_format.clone(),
        }
    }
}

impl BenchmarkWorker {
    pub fn new(id: usize, payload: &WorkerPayload) -> BenchmarkWorker {
        BenchmarkWorker {
            id,
            docker_manager: None,
            thread_handle: None,
            benchmark_dir: payload.benchmark_dir.clone(),
            test_sets: payload.test_sets.clone(),
            current_run: payload.current_run,
            local_test_image_dir: payload.local_test_image_dir.clone(),
            docker_test_image_dir: payload.docker_test_image_dir.clone(),
            current_worker_id: payload.current_worker_id,
            current_output_dir: payload.current_output_dir.clone(),
            current_result_dir: payload.current_result_dir.clone(),
            current_image_name: payload.current_image_name.clone(),
            current_image_file_path: payload.current_image_file_path.clone(),
            current_image_format: payload.current_image_format.clone(),
        }
    }

    pub fn run(&mut self, benchmark: Arc<Mutex<dyn Benchmark>>) {
        let payload = WorkerPayload::from(self.clone());
        let benchmark_clone = Arc::clone(&benchmark);
        self.thread_handle = Some(std::thread::spawn(move || {
            let mut benchmark = benchmark_clone.lock().unwrap();
            benchmark.run(payload);
        }));
    }
}

#[derive(Debug, Clone)]
pub struct WorkerPayload {
    pub benchmark_dir: String,
    pub test_sets: Vec<String>,
    pub current_run: usize,
    pub local_test_image_dir: String,
    pub docker_test_image_dir: String,
    pub current_worker_id: usize,
    pub current_output_dir: String,
    pub current_result_dir: String,
    pub current_image_name: String, // "kodim06"
    pub current_image_file_path: String, // "kodim/kodim06.png"
    pub current_image_format: ImageFormat,
}

impl WorkerPayload { 
    pub fn get_output_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/output", benchmark_dir, current_run)
    }

    pub fn get_result_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/results", benchmark_dir, current_run)
    }

    pub fn get_output_path_for(&self, file_path: String) -> String {
        format!("{}/{}/output/{}", self.benchmark_dir, self.current_run, file_path)
    }

    pub fn get_result_path_for(&self, file_path: String) -> String {
        format!("{}/{}/results/{}", self.benchmark_dir, self.current_run, file_path)
    }
}

impl From<BenchmarkWorker> for WorkerPayload {
    fn from(worker: BenchmarkWorker) -> Self {
        WorkerPayload {
            benchmark_dir: worker.benchmark_dir.clone(),
            test_sets: worker.test_sets.clone(),
            current_run: worker.current_run,
            local_test_image_dir: worker.local_test_image_dir.clone(),
            docker_test_image_dir: worker.docker_test_image_dir.clone(),
            current_worker_id: worker.id,
            current_output_dir: worker.current_output_dir.clone(),
            current_result_dir: worker.current_result_dir.clone(),
            current_image_name: worker.current_image_name.clone(),
            current_image_file_path: worker.current_image_file_path.clone(),
            current_image_format: worker.current_image_format.clone(),
        }
    }
}


#[derive(Debug)]
pub struct Benchmarker {
    pub benchmark_dir: String,
    pub test_sets: Vec<String>,
    pub current_run: usize,
    pub local_test_image_dir: String,
    pub docker_test_image_dir: String,
    pub num_workers: usize,
    pub workers: Vec<BenchmarkWorker>,
    pub current_worker_id: usize,
}

impl Clone for Benchmarker {
    fn clone(&self) -> Self {
        Benchmarker {
            benchmark_dir: self.benchmark_dir.clone(),
            test_sets: self.test_sets.clone(),
            current_run: self.current_run,
            local_test_image_dir: self.local_test_image_dir.clone(),
            docker_test_image_dir: self.docker_test_image_dir.clone(),
            num_workers: self.num_workers,
            workers: Vec::new(),
            current_worker_id: self.current_worker_id,
        }
    }
}

impl From<Benchmarker> for WorkerPayload {
    fn from(benchmarker: Benchmarker) -> Self {
        WorkerPayload {
            benchmark_dir: benchmarker.benchmark_dir.clone(),
            test_sets: benchmarker.test_sets.clone(),
            current_run: benchmarker.current_run,
            local_test_image_dir: benchmarker.local_test_image_dir.clone(),
            docker_test_image_dir: benchmarker.docker_test_image_dir.clone(),
            current_worker_id: benchmarker.current_worker_id,
            current_output_dir: String::new(),
            current_result_dir: String::new(),
            current_image_name: String::new(),
            current_image_file_path: String::new(),
            current_image_format: ImageFormat::Unsupported,
        }
    }
}

impl Benchmarker {
    pub fn new(benchmark_dir: String, local_test_image_dir: String, docker_test_image_dir: String, num_workers: usize) -> Benchmarker {
        let dir = PathBuf::from(benchmark_dir.clone());
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
        }

        let mut b = Benchmarker {
            benchmark_dir: benchmark_dir.clone(),
            test_sets: Benchmarker::get_all_test_set_names(local_test_image_dir.clone()),
            current_run: Benchmarker::get_current_run(benchmark_dir),
            local_test_image_dir: local_test_image_dir.clone(),
            docker_test_image_dir: docker_test_image_dir.clone(),
            num_workers,
            workers: Vec::new(), 
            current_worker_id: 0,
        };
        let config = Config::default();
        for x in 0..b.num_workers {
            println!("Spawning worker {}", x);
            let payload = WorkerPayload::from(b.clone());
            let mut worker = BenchmarkWorker::new(x, &payload);
            println!("Setting up docker");
            let mut docker_manager = DockerManager::new(&config.docker_file_path, x);
            let res = docker_manager.setup(worker.id);
            match res {
                Ok(_) => println!("Setup complete"),
                Err(err) => println!("Error {}", err),
            }
            worker.docker_manager = Some(docker_manager);
            b.workers.push(worker);
        }
        b
    }

    fn get_next_worker_id(&mut self) -> usize {
        let id = self.current_worker_id;
        self.current_worker_id += 1;
        if self.current_worker_id >= self.num_workers {
            self.current_worker_id = 0;
        }
        id
    }

    pub fn get_output_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/output", benchmark_dir, current_run)
    }

    pub fn get_result_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/results", benchmark_dir, current_run)
    }

    pub fn get_output_path_for(&self, file_path: String) -> String {
        format!("{}/{}/output/{}", self.benchmark_dir, self.current_run, file_path)
    }

    pub fn get_result_path_for(&self, file_path: String) -> String {
        format!("{}/{}/results/{}", self.benchmark_dir, self.current_run, file_path)
    }

    pub fn get_current_run(benchmark_dir: String) -> usize {
        let mut current_run = 0;
        let path = PathBuf::from(benchmark_dir.clone());
        if path.exists() {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                println!("File {}", file_name);
                // if file name is not a number, skip it, for temp/ directory in non-temp mode
                if file_name.parse::<usize>().is_err() {
                    continue;
                }
                println!("Found file {}", file_name);
                let run = file_name.parse::<usize>().unwrap();
                println!("Found run {}", run);
                if run >= current_run {
                    current_run = run + 1;
                }
            }
        }
        println!("Current run: {}", current_run);
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

    pub fn get_next_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.get_next_worker_id();
        &mut self.workers[id] as &mut BenchmarkWorker
    }

    pub fn get_current_worker(&mut self) -> &mut BenchmarkWorker {
        let id = self.current_worker_id;
        &mut self.workers[id] as &mut BenchmarkWorker
    }

    pub fn run_benchmark(&mut self, benchmark: Arc<Mutex<dyn Benchmark>>) {
        self.current_run = Benchmarker::get_current_run(self.benchmark_dir.clone());
        // make current run directory
        let current_run_dir = format!("{}/{}", self.benchmark_dir, self.current_run);
        if !PathBuf::from(current_run_dir.clone()).exists() {
            println!("Creating current run dir {}", current_run_dir.clone());
            fs::create_dir_all(current_run_dir.clone()).unwrap();
        }
        println!("Running benchmark for run {}", self.current_run);
        let test_sets = self.test_sets.clone();
        for test_set in &test_sets {
            println!("Running benchmark for test set {}", test_set);
            let test_set_path = format!("{}/{}", self.local_test_image_dir, test_set);
            println!("Test set path: {}", test_set_path.clone());
            for entry in fs::read_dir(test_set_path).unwrap() {
                let output_path_for = self.get_output_path_for(test_set.clone());
                let result_path_for = self.get_result_path_for(test_set.clone());
                let worker = self.get_next_worker();

                let entry = entry.unwrap();
                let path = entry.path();
                println!("Image path: {}", path.to_str().unwrap());

                worker.current_image_file_path = path.to_str().unwrap().to_string();
                println!("Image file path: {}", worker.current_image_file_path.clone());
                worker.current_image_name = path.file_name().unwrap().to_str().unwrap().to_string();
                println!("Image name: {}", worker.current_image_name.clone());
                // remove file extension
                worker.current_image_name = worker.current_image_name.as_str().split(".").collect::<Vec<&str>>()[0].to_string();
                println!("Image name: {}", worker.current_image_name.clone());

                worker.current_output_dir = output_path_for;
                println!("Output dir: {}", worker.current_output_dir.clone());
                worker.current_result_dir = result_path_for;
                println!("Result dir: {}", worker.current_result_dir.clone());
                worker.current_image_format = ImageFormat::from_file_name(&worker.current_image_file_path);

                match worker.current_image_format {
                    ImageFormat::Unsupported => continue,
                    _ => (),
                }

                println!("Image format: {}", worker.current_image_format.to_string());

                if !PathBuf::from(worker.current_output_dir.clone()).exists() {
                    println!("Creating output dir {}", worker.current_output_dir.clone());
                    fs::create_dir_all(worker.current_output_dir.clone()).unwrap();
                }
                if !PathBuf::from(worker.current_result_dir.clone()).exists() {
                    println!("Creating result dir {}", worker.current_result_dir.clone());
                    fs::create_dir_all(worker.current_result_dir.clone()).unwrap();
                }

                println!("Running benchmark for image {}", worker.current_image_file_path.clone());
                worker.run(benchmark.clone());
            }
        }
    }

    pub fn teardown(&mut self) {
        for worker in &mut self.workers {
            worker.docker_manager.as_ref().unwrap().teardown().unwrap();
        }
    }
}

pub trait Benchmark: Sync + Send {
    fn run(&mut self, payload: WorkerPayload);
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

        let csv_writer = ImageFileDataCSVWriter::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();
    }
}*/

pub struct JXLCompressionBenchmark<'a> {
    benchmarker: &'a mut Benchmarker,
    to_format: ImageFormat,
}

impl<'a> JXLCompressionBenchmark<'a> {
    pub fn new(
        benchmarker: &'a mut Benchmarker,
    ) -> JXLCompressionBenchmark<'a> {
        JXLCompressionBenchmark {
            benchmarker,
            to_format: ImageFormat::JpegXl,
        }
    }

    fn execute_cjxl(
        &mut self,
        input_file: String,
        output_file: String,
        distance: f64,
        effort: u32,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        let distance = format!("--distance={}", distance);
        let effort = format!("--effort={}", effort);
        let args = vec![
            input_file.as_str(),
            output_file.as_str(),
            distance.as_str(),
            effort.as_str(),
        ];

        self.benchmarker.get_current_worker()
            .docker_manager.as_ref()
            .expect("Docker manager not set")
            .execute_in_container("../libjxl/build/tools/cjxl", args)
    }

    fn retrieve_file(&mut self, file_path: String, dest_path: String) -> Result<String, Box<dyn Error>> {
        self.benchmarker.get_current_worker()
            .docker_manager.as_ref()
            .expect("Docker manager not set")
            .retrieve_file(file_path, dest_path)
    }
}

impl<'a> Benchmark for JXLCompressionBenchmark<'a> {
    fn run(&mut self, payload: WorkerPayload) {
        println!("Running Compression Benchmark for image {}", payload.current_image_name);
        let out_orig_path = payload.get_output_path_for("orig".to_string());
        println!("Output orig path: {}", out_orig_path.clone());
        let out_comp_path = payload.get_output_path_for("comp".to_string());
        println!("Output comp path: {}", out_comp_path.clone());
        if !PathBuf::from(out_orig_path.clone()).exists() {
            fs::create_dir_all(out_orig_path.clone()).unwrap();
        }
        if !PathBuf::from(out_comp_path.clone()).exists() {
            fs::create_dir_all(out_comp_path.clone()).unwrap();
        }
        let res_orig_path = payload.get_result_path_for("orig".to_string());
        println!("Result orig path: {}", res_orig_path.clone());
        let res_comp_path = payload.get_result_path_for("comp".to_string());
        println!("Result comp path: {}", res_comp_path.clone());
        if !PathBuf::from(res_orig_path.clone()).exists() {
            fs::create_dir_all(res_orig_path.clone()).unwrap();
        }
        if !PathBuf::from(res_comp_path.clone()).exists() {
            fs::create_dir_all(res_comp_path.clone()).unwrap();
        }

        let file_path = format!(
            "{}/{}",
            payload.docker_test_image_dir,
            payload.current_image_file_path.split(&format!("{}/", &payload.local_test_image_dir)).collect::<Vec<&str>>()[1]
        );
        println!("File path: {}", file_path.clone());

        let image_reader = ImageReader::new(payload.current_image_file_path.clone().to_string());
        let image_file_data = image_reader.file_data;
        let result_file = format!("{}/{}.csv", res_orig_path, payload.current_image_name.clone());
        println!("Result file: {}", result_file.clone());
        let csv_writer = ImageFileDataCSVWriter::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();

        let distances = vec![0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 15.0, 25.0];
        let efforts = (1..=9).collect::<Vec<u32>>();

        for distance in distances {
            for effort in &efforts {
                let comp_image_name = format!("{}-{}-{}.{}", payload.current_image_name, distance, effort, self.to_format.to_string());
                println!("Compressed image name: {}", comp_image_name.clone());

                let result = self
                    .execute_cjxl(
                        file_path.to_string(),
                        comp_image_name.clone(),
                        distance,
                        *effort,
                    )
                    .unwrap();

                match result {
                    Ok(output) => {
                        println!("Output: {}", output);

                        println!("Retrieving compressed image");
                        let src_path = format!("/temp/{}", comp_image_name);
                        println!("Source path: {}", src_path.clone());
                        let dest_path = format!("{}/{}", out_comp_path, comp_image_name);
                        println!("Dest path: {}", dest_path.clone());
                        self.retrieve_file(src_path, dest_path).unwrap();

                        println!("Reading compressed image");
                        let image_reader = ImageReader::new(format!("{}/{}", out_comp_path, comp_image_name));
                        let image_file_data = image_reader.file_data;
                        let result_file = format!("{}/{}-{}-{}.csv", res_comp_path, payload.current_image_name, distance, effort);
                        println!("Result file: {}", result_file.clone());
                        let csv_writer = ImageFileDataCSVWriter::new();
                        csv_writer.write_csv_header(&result_file).unwrap();
                        csv_writer
                            .write_csv(&vec![image_file_data], &result_file)
                            .unwrap();
                    }
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            }
        }
    }
}
