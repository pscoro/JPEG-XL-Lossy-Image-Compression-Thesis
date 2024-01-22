use clap::Parser;
use clap_derive::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use benchmark_jpegxl::benchmark::{
    Benchmarker, /*CollectImageMetadataBenchmark,*/ JXLCompressionBenchmark, Benchmark,
};
use benchmark_jpegxl::config::Config;

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
    let benchmark_path = config.benchmark_dir_path.to_owned()
        + match args.temp {
            true => "/temp",
            false => match config.use_temp_dir {
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
        config.local_test_image_dir_path.to_owned(),
        config.docker_test_image_dir_path.to_owned(),
        8,
    );

    //    println!("Running collect image metadata benchmark");
    //    let collect_image_metadata_benchmark = CollectImageMetadataBenchmark {};
    //    benchmarker.run_benchmark(&collect_image_metadata_benchmark);

    println!("Running JPEG-XL Compression benchmark");
    let jpegxl_compression_benchmark = JXLCompressionBenchmark::new(&mut benchmarker);
    let benchmark = Arc::new(Mutex::new(jpegxl_compression_benchmark));
    benchmarker.run_benchmark(benchmark);

    benchmarker.teardown();
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
