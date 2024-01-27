use clap::Parser;
use clap_derive::Parser;
use std::fs;

use benchmark_jpegxl::benchmark::{
    Benchmarker, /*CollectImageMetadataBenchmark,*/ JXLCompressionBenchmark,
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
    benchmarker.run_benchmark::<JXLCompressionBenchmark>();

    for worker in &mut benchmarker.workers {
        println!("Waiting for worker {} to finish", worker.id);
        if let Some(thread_handle) = worker.thread_handle.take() {
            thread_handle.join().unwrap();
        } else {
            panic!("Worker {} has no thread handle", worker.id);
        }
    }
    benchmarker.teardown();
}
