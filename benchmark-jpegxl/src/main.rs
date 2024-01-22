use clap::Parser;
use clap_derive::Parser;
use std::fs;
use std::path::PathBuf;

use benchmark_jpegxl::benchmark::{
    Benchmarker, /*CollectImageMetadataBenchmark,*/ JXLCompressionBenchmark,
};
use benchmark_jpegxl::config::Config;
use benchmark_jpegxl::docker_manager::DockerManager;

/// Arguments
/// `--clean, -c` - Clean all benchmark files
/// `--temp, -t` - Use temp directory for benchmark files
#[derive(Parser)]
#[clap(name = "Benchmark JPEG-XL")]
struct Args {
    #[arg(short, long)]
    clean: bool,
    #[arg(short, long)]
    temp: bool,
}

fn main() {
    println!("Benchmark JPEG-XL");

    println!("Parsing arguments");
    let args = Args::parse();

    println!("Setting up config");
    let config = Config::default();

    println!("Setting up benchmark directory");
    let benchmark_path = config.BENCHMARK_DIR_PATH.to_owned()
        + match args.temp {
            true => "/temp",
            false => match config.USE_TEMP_DIR {
                true => "/temp",
                false => "",
            },
        };

    match args.clean {
        true => {
            println!("Cleaning benchmark directory at {}", benchmark_path);
            fs::remove_dir_all(benchmark_path.clone()).unwrap();
        }
        false => {}
    }

    println!("Creating benchmark directory at {}", benchmark_path);
    fs::create_dir_all(benchmark_path.clone()).unwrap();

    println!("Setting up benchmarker");
    let mut benchmarker = Benchmarker::new(
        benchmark_path,
        config.LOCAL_TEST_IMAGE_DIR_PATH.to_owned(),
        config.DOCKER_TEST_IMAGE_DIR_PATH.to_owned(),
        8,
    );

    //    println!("Running collect image metadata benchmark");
    //    let collect_image_metadata_benchmark = CollectImageMetadataBenchmark {};
    //    benchmarker.run_benchmark(&collect_image_metadata_benchmark);

    println!("Running JPEG-XL Compression benchmark");
    let jpegxl_compression_benchmark = JXLCompressionBenchmark::new(&benchmarker);
    benchmarker.run_benchmark(&jpegxl_compression_benchmark);

    let res = docker_manager.teardown();
    match res {
        Ok(_) => println!("Teardown complete"),
        Err(err) => println!("Error {}", err),
    }
}

fn all_dirs_in(path: &str) -> Vec<String> {
    let mut test_sets = Vec::new();
    let path = PathBuf::from(path);
    if path.exists() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                let dir_name = entry.file_name().into_string().unwrap();
                test_sets.push(dir_name);
            }
        }
    }
    println!("Found test sets {:?}", test_sets);
    test_sets
}
