use crate::config::Config;
use crate::csv_writer::{
    CSVReader, CSVWriter, ComparisonResult, ComparisonResultCSVWriter, ImageFileDataCSVWriter,
};
use crate::docker_manager::DockerManager;
use crate::image_reader::{ImageFileData, ImageFormat, ImageReader};

use std::fs;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;

pub trait Benchmark: Sync + Send {
    fn run(docker_manager: DockerManager, payload: &WorkerPayload);
}

pub struct JXLCompressionBenchmark {}

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

#[derive(Debug)]
pub struct BenchmarkWorker {
    pub id: usize,
    pub docker_manager: Option<DockerManager>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
    pub payload: Option<WorkerPayload>,
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
    pub current_image_name: String,      // "kodim06"
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

impl BenchmarkWorker {
    pub fn new(id: usize, payload: &WorkerPayload) -> BenchmarkWorker {
        BenchmarkWorker {
            id,
            docker_manager: None,
            thread_handle: None,
            payload: Some(payload.clone()),
        }
    }

    pub fn run<T: Benchmark + 'static>(&mut self) {
        println!("Running worker {}", self.id);
        let payload = self.payload.as_ref().unwrap().clone();
        if self.thread_handle.is_some() {
            self.thread_handle.take().unwrap().join().unwrap();
        }
        let docker = self.docker_manager.as_mut().unwrap().clone();
        self.thread_handle = Some(std::thread::spawn(move || {
            T::run(docker, &payload);
        }));
        println!("Thread spawned");
    }
}

impl WorkerPayload {
    pub fn get_output_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/output", benchmark_dir, current_run)
    }

    pub fn get_result_dir(benchmark_dir: String, current_run: usize) -> String {
        format!("{}/{}/results", benchmark_dir, current_run)
    }

    pub fn get_output_path_for(&self, file_path: String) -> String {
        format!(
            "{}/{}/output/{}",
            self.benchmark_dir, self.current_run, file_path
        )
    }

    pub fn get_result_path_for(&self, file_path: String) -> String {
        format!(
            "{}/{}/results/{}",
            self.benchmark_dir, self.current_run, file_path
        )
    }
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

impl Benchmarker {
    pub fn new(
        benchmark_dir: String,
        local_test_image_dir: String,
        docker_test_image_dir: String,
        num_workers: usize,
    ) -> Benchmarker {
        let dir = PathBuf::from(benchmark_dir.clone());
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
        }

        let mut b = Benchmarker {
            benchmark_dir: benchmark_dir.clone(),
            test_sets: Benchmarker::get_all_test_set_names(local_test_image_dir.clone()),
            current_run: Benchmarker::get_current_run(benchmark_dir.clone()),
            local_test_image_dir: local_test_image_dir.clone(),
            docker_test_image_dir: docker_test_image_dir.clone(),
            num_workers,
            workers: Vec::new(),
            current_worker_id: 0,
        };
        let config = Config::default();
        for x in 0..b.num_workers {
            println!("Spawning worker {}", x);
            let payload = WorkerPayload {
                benchmark_dir: benchmark_dir.clone(),
                test_sets: b.test_sets.clone(),
                current_run: b.current_run,
                local_test_image_dir: local_test_image_dir.clone(),
                docker_test_image_dir: docker_test_image_dir.clone(),
                current_worker_id: x,
                current_output_dir: Benchmarker::get_output_dir(
                    benchmark_dir.clone(),
                    b.current_run,
                ),
                current_result_dir: Benchmarker::get_result_dir(
                    benchmark_dir.clone(),
                    b.current_run,
                ),
                current_image_name: "".to_string(),
                current_image_file_path: "".to_string(),
                current_image_format: ImageFormat::Unsupported,
            };
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
        format!(
            "{}/{}/output/{}",
            self.benchmark_dir, self.current_run, file_path
        )
    }

    pub fn get_result_path_for(&self, file_path: String) -> String {
        format!(
            "{}/{}/results/{}",
            self.benchmark_dir, self.current_run, file_path
        )
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
        println!("Local test image dir: {}", local_test_image_dir.clone());
        if path.exists() {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let test_set_dir_name = path.file_name().unwrap().to_str().unwrap();
                println!("Test set dir name: {}", test_set_dir_name);
                // if test_set_dir_name starts with a "." or is not a directory, skip it
                if test_set_dir_name.starts_with(".") || !path.is_dir() {
                    continue;
                }
                println!("Found test set {}", test_set_dir_name);
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

    pub fn wait_for_workers(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread_handle) = worker.thread_handle.take() {
                thread_handle.join().unwrap();
            } else {
                panic!("Worker {} has no thread handle", worker.id);
            }
        }
    }

    pub fn run_benchmark<T: Benchmark + 'static>(&mut self) {
        println!("Running benchmark");
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

                worker.payload.as_mut().unwrap().current_image_file_path =
                    path.to_str().unwrap().to_string();
                println!(
                    "Image file path: {}",
                    worker
                        .payload
                        .as_ref()
                        .unwrap()
                        .current_image_file_path
                        .clone()
                );
                worker.payload.as_mut().unwrap().current_image_name =
                    path.file_name().unwrap().to_str().unwrap().to_string();
                println!(
                    "Image name: {}",
                    worker.payload.as_ref().unwrap().current_image_name.clone()
                );

                // remove file extension
                worker.payload.as_mut().unwrap().current_image_name = worker
                    .payload
                    .as_mut()
                    .unwrap()
                    .current_image_name
                    .as_str()
                    .split(".")
                    .collect::<Vec<&str>>()[0]
                    .to_string();
                println!(
                    "Image name: {}",
                    worker.payload.as_ref().unwrap().current_image_name.clone()
                );

                worker.payload.as_mut().unwrap().current_output_dir = output_path_for;
                println!(
                    "Output dir: {}",
                    worker.payload.as_ref().unwrap().current_output_dir.clone()
                );
                worker.payload.as_mut().unwrap().current_result_dir = result_path_for;
                println!(
                    "Result dir: {}",
                    worker.payload.as_ref().unwrap().current_result_dir.clone()
                );
                worker.payload.as_mut().unwrap().current_image_format = ImageFormat::from_file_name(
                    &worker.payload.as_ref().unwrap().current_image_file_path,
                );

                match worker.payload.as_ref().unwrap().current_image_format {
                    ImageFormat::Unsupported => continue,
                    _ => (),
                }

                println!(
                    "Image format: {}",
                    worker
                        .payload
                        .as_ref()
                        .unwrap()
                        .current_image_format
                        .to_string()
                );

                if !PathBuf::from(worker.payload.as_ref().unwrap().current_output_dir.clone())
                    .exists()
                {
                    println!(
                        "Creating output dir {}",
                        worker.payload.as_ref().unwrap().current_output_dir.clone()
                    );
                    fs::create_dir_all(worker.payload.as_ref().unwrap().current_output_dir.clone())
                        .unwrap();
                }
                if !PathBuf::from(worker.payload.as_ref().unwrap().current_result_dir.clone())
                    .exists()
                {
                    println!(
                        "Creating result dir {}",
                        worker.payload.as_ref().unwrap().current_result_dir.clone()
                    );
                    fs::create_dir_all(worker.payload.as_ref().unwrap().current_result_dir.clone())
                        .unwrap();
                }

                println!(
                    "Running benchmark for image {}",
                    worker
                        .payload
                        .as_ref()
                        .unwrap()
                        .current_image_file_path
                        .clone()
                );
                worker.run::<T>();
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

        let csv_writer = ImageFileDataCSVWriter::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();
    }
}*/

impl Benchmark for JXLCompressionBenchmark {
    fn run(docker_manager: DockerManager, payload: &WorkerPayload) {
        println!(
            "Running Compression Benchmark for image {}",
            payload.current_image_name
        );
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
            payload
                .current_image_file_path
                .split(&format!("{}/", &payload.local_test_image_dir))
                .collect::<Vec<&str>>()[1]
        );
        println!("File path: {}", file_path.clone());

        let image_reader = ImageReader::new(payload.current_image_file_path.clone().to_string());
        let image_file_data = image_reader.file_data;
        let result_file = format!("{}/results.csv", res_orig_path,);
        println!("Result file: {}", result_file.clone());
        let csv_writer = ImageFileDataCSVWriter::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![image_file_data], &result_file)
            .unwrap();

        let distances = vec![0.0, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 15.0, 25.0];
        let efforts = (1..=9).collect::<Vec<u32>>();
        let test_set = payload
            .current_image_file_path
            .split("/")
            .collect::<Vec<&str>>()[2];
        let out_test_set_path = format!("{}/{}", out_comp_path, test_set);
        if !PathBuf::from(out_test_set_path.clone()).exists() {
            fs::create_dir_all(out_test_set_path.clone()).unwrap();
        }
        let res_test_set_path = format!("{}/{}", out_comp_path, test_set);
        if !PathBuf::from(res_test_set_path.clone()).exists() {
            fs::create_dir_all(res_test_set_path.clone()).unwrap();
        }

        for distance in distances {
            for effort in &efforts {
                let comp_image_name = format!(
                    "{}/{}-{}-{}.{}",
                    test_set,
                    payload.current_image_name,
                    distance,
                    effort,
                    ImageFormat::JpegXl.to_string()
                );
                println!("Compressed image name: {}", comp_image_name.clone());

                let result = docker_manager
                    .execute_cjxl(
                        file_path.to_string().clone(),
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
                        docker_manager
                            .retrieve_file(src_path.clone(), dest_path)
                            .unwrap();

                        println!("Reading compressed image");
                        let image_reader =
                            ImageReader::new(format!("{}/{}", out_comp_path, comp_image_name));
                        let image_file_data = image_reader.file_data;
                        let result_file = format!("{}/results.csv", res_comp_path,);
                        println!("Result file: {}", result_file.clone());
                        let csv_writer = ImageFileDataCSVWriter::new();
                        csv_writer.write_csv_header(&result_file).unwrap();
                        csv_writer
                            .write_csv(&vec![image_file_data.clone()], &result_file)
                            .unwrap();

                        println!("Comparing images");
                        JXLCompressionBenchmark::compare_images(
                            &image_file_data,
                            &out_comp_path,
                            &res_orig_path,
                            &res_comp_path,
                            &docker_manager,
                            &file_path,
                            &src_path,
                        );
                    }
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            }
        }
    }
}

impl JXLCompressionBenchmark {
    fn compare_images(
        comp_image_data: &ImageFileData,
        out_comp_path: &str,
        res_orig_path: &str,
        res_comp_path: &str,
        docker_manager: &DockerManager,
        docker_input_path: &str,
        docker_output_path: &str,
    ) {
        let config = Config::default();
        let csv_writer = ImageFileDataCSVWriter::new();
        let orig_entry = csv_writer
            .find_entry(
                format!("{}/results.csv", res_orig_path).as_str(),
                0,
                format!("{}.png", &comp_image_data.jxl_orig_image_name).as_str(),
            )
            .unwrap();
        println!(
            "Comparing {} to {}",
            comp_image_data.image_name, orig_entry.image_name
        );

        // Comparison calculations
        // Original file size to compressed file size ratio
        let orig_file_size = orig_entry.file_size as f64;
        let comp_file_size = comp_image_data.file_size as f64;
        let file_size_ratio = orig_file_size / comp_file_size;

        // Raw image size to compressed file size ratio
        let orig_raw_size = orig_entry.raw_size as f64;
        let comp_file_size = comp_image_data.file_size as f64;
        let raw_size_ratio = orig_raw_size / comp_file_size;

        // MSE
        let orig_image_path = orig_entry.file_path.clone();
        let comp_image_path = comp_image_data.file_path.clone();
        let mse = ImageReader::calculate_mse(&orig_image_path, &comp_image_path);

        // PSNR
        let psnr = ImageReader::calculate_psnr(mse, 255.0);

        // SSIM
        let orig_image_path = orig_entry.file_path.clone();
        let comp_image_path = comp_image_data.file_path.clone();
        // $ magick compare -metric SSIM orig.png comp.png diff.png
        println!("Running command: magick compare -metric SSIM {} {} {}/diff.png", orig_image_path, comp_image_path, out_comp_path);
        let result = Command::new("magick")
            .arg("compare")
            .arg("-metric")
            .arg("SSIM")
            .arg(orig_image_path)
            .arg(comp_image_path)
            .arg("null:")
            .output();
        println!("Result: {:?}", result);
        let ssim = result
            .unwrap()
            .stderr
            .lines()
            .next()
            .unwrap()
            .expect("Error calculating SSIM")
            .parse::<f64>()
            .unwrap();

        // MS-SSIM
        // TODO: Implement MS-SSIM

        // Butteraugli
        let result = docker_manager.execute_butteraugli(
            docker_input_path.to_string().clone(),
            docker_output_path.to_string().clone(),
        );
        let output = result.unwrap().unwrap();
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

        // SSIMULACRA2
        let result = docker_manager.execute_ssimulacra2(
            docker_input_path.to_string().clone(),
            docker_output_path.to_string().clone(),
        );
        let output = result.unwrap().unwrap();
        println!("SSIMULACRA2 output: {}", output.clone());
        let ssimulacra2 = output.lines().next().unwrap().parse::<f64>().unwrap();

        let comparison_result = ComparisonResult {
            orig_image_name: orig_entry.image_name.clone(),
            comp_image_name: comp_image_data.image_name.clone(),
            distance: comp_image_data.jxl_distance.into(),
            effort: comp_image_data.jxl_effort.into(),
            orig_file_size: orig_entry.file_size as u64,
            comp_file_size: comp_image_data.file_size as u64,
            orig_raw_size: orig_entry.raw_size as u64,
            comp_raw_size: comp_image_data.raw_size as u64,
            file_size_ratio,
            raw_size_ratio,
            mse,
            psnr,
            ssim,
            ms_ssim: 0.0,
            butteraugli,
            butteraugli_pnorm: pnorm,
            ssimulacra2,
        };

        let result_file = format!("{}/comparisons.csv", res_comp_path,);

        let csv_writer = ComparisonResultCSVWriter::new();
        csv_writer.write_csv_header(&result_file).unwrap();
        csv_writer
            .write_csv(&vec![comparison_result], &result_file)
            .unwrap();
    }
}
