import argparse
import csv
import os
import glob
import subprocess

from enum import Enum
from PIL import Image
from termcolor import colored

DIFF_TOOL = "compare"
EFFORT_LEVELS = 10

class ImageHelper:
    @staticmethod
    def get_image_dims(image_path):
        with Image.open(image_path) as img:
            return f"{img.width} x {img.height}"


    @staticmethod
    def get_image_size(image_path):
        return os.path.getsize(image_path)


    @staticmethod
    def get_image_format(image_path):
        return image_path.split('.')[-1]


class SupportedImageExt(Enum):
    JXL = "*.jxl"
    PNG = "*.png"
    JPG = "*.jpg"


class Command:
    def __init__(self, cmd):
        self.cmd = cmd


    def to_string(self):
        return self.cmd


class ImageCompressionData:
    def __init__(
            self,
            input_image_path,
            compressed_image_path,
            compression_effort,
            ):
        self.input_image_path = input_image_path
        self.compressed_image_path = compressed_image_path
        self.compression_effort = compression_effort

        self.image_dims = ImageHelper.get_image_dims(input_image_path)
        self.input_image_size = ImageHelper.get_image_size(input_image_path)
        self.input_image_format = ImageHelper.get_image_format(input_image_path)
        self.compressed_image_size = ImageHelper.get_image_size(compressed_image_path)
        self.compressed_image_format = ImageHelper.get_image_format(compressed_image_path)
        self.delta_size = self.compressed_image_size - self.input_image_size
        self.percent_of_orig = self.compressed_image_size / self.input_image_size * 100


    @staticmethod
    def get_col_names():
        return [
            "Input Image Path",
            "Compressed Image Path",
            "Compression Effort",
            "Image Dims",
            "Input Image Size",
            "Input Image Format",
            "Compressed Image Size",
            "Compressed Image Format",
            "Delta Size",
            "Percent Of Orig",
                ]


class Table:
    def __init__(self, title, col_names=None, data=None):
        if col_names is None:
            col_names = []
        if data is None:
            data = []

        self.title = title
        self.col_names = col_names
        self.data = data


    def to_csv(self, results_path):
        csv_data = [self.col_names] + self.data
        csv_file = os.path.join(results_path, self.title + ".csv")
        with open(csv_file, mode='w', newline='') as file:
            writer = csv.writer(file)
            writer.writerows(csv_data)
        print(f"CSV file '{csv_file}' has been created.")

    
    def create_col(self, col_name, col_data):
        self.col_names.append(col_name)
        i = 0
        for row in self.data:
            row.append(col_data[i])
            i += 1


    def create_row(self, row_data):
        self.data.append(row_data)


    def create_row_from_compression_data(self, cd):
        row_data = [
            cd.input_image_path,
            cd.compressed_image_path,
            cd.compression_effort,
            cd.image_dims,
            cd.input_image_size,
            cd.input_image_format,
            cd.compressed_image_size,
            cd.compressed_image_format,
            cd.delta_size,
            cd.percent_of_orig,
                ]
        self.data.append(row_data)


class JXLTester:

    def __init__(self):
        self.args = None
        self.data = {}

        # e.g. /path/to/test-images/kodak/
        self.dataset_paths = []

        # e.g. /path/to/test-scripts/out/
        self.root_output_dir = os.path.abspath("out")

        # e.g. /path/to/test-scripts/results/
        self.root_results_dir = os.path.abspath("results")
        
        # e.g. 17
        self.curr_output_num = -1

        # e.g. /path/to/test-scripts/out/17/
        self.curr_output_dir = ""

        # e.g. /path/to/test-scripts/results/17/
        self.curr_results_dir = ""


    def create_curr_run_dirs(self):
        if not os.path.exists(self.root_output_dir):
            os.makedirs(self.root_output_dir, exist_ok=True)

        if not os.path.exists(self.root_results_dir):
            os.makedirs(self.root_results_dir, exist_ok=True)

        max_dir_number = -1

        for dir_name in os.listdir(self.root_output_dir):
            try:
                dir_number = int(dir_name)
                max_dir_number = max(max_dir_number, dir_number)
            except Exception:
                pass

        self.curr_output_num = max_dir_number + 1
        new_dir_name = str(self.curr_output_num)
        self.curr_output_dir = os.path.join(self.root_output_dir, new_dir_name)
        self.curr_results_dir = os.path.join(self.root_results_dir, new_dir_name)

        os.makedirs(self.curr_output_dir, exist_ok=True)
        os.makedirs(self.curr_results_dir, exist_ok=True)


    def setup_paths(self, dataset_path, dir_name):
        assert os.path.isdir(dataset_path), "No image dataset found."

        dataset_name = os.path.split(dataset_path)[1]

        output_dir_path = os.path.join(self.curr_output_dir, dataset_name, dir_name)
        if not os.path.isdir(output_dir_path):
            os.makedirs(output_dir_path, exist_ok=True)

        results_dir_path = os.path.join(self.curr_results_dir, dataset_name, dir_name)
        if not os.path.isdir(results_dir_path):
            os.makedirs(results_dir_path, exist_ok=True)


        return output_dir_path, results_dir_path

    
    def get_images_in_dir(self, dir_name, image_type=SupportedImageExt.JXL):
        images = glob.glob(os.path.abspath(os.path.join(dir_name, image_type.value)))
        assert len(images) > 0, f"No images found in {dir_name}."
        return images


    def get_dataset_name(self, dataset_path):
        return os.path.split(dataset_path)[1]


    def start_test_run(self, dataset_paths):
        self.create_curr_run_dirs()
        for dataset_path in dataset_paths:
            compress_output_path, compress_results_path = self.setup_paths(dataset_path, "compress")
            compression_table = self.compress_from_png(dataset_path, compress_output_path)

            compression_table.to_csv(compress_results_path)

            decompress_output_path, decompress_results_path = self.setup_paths(dataset_path, "decompress")
            run_output_files = self.decompress_to_png(compress_output_path, decompress_output_path)

            print("List of decompressed files:")
            print(run_output_files)

            compare_output_path, compare_results_path = self.setup_paths(dataset_path, "comparison")
            self.compare_images(dataset_path, decompress_output_path, compare_output_path)
    

    def parse_arguments(self):
        parser = argparse.ArgumentParser(description="Process png images in a given directory.")
        parser.add_argument('-t', '--test_set', required=False, type=str, help='Directory containing png images')
        parser.add_argument('-ts', '--test_sets', required=False, type=str, help='Directories containing png images')
        #parser.add_argument('-i', '--include', required=False, type=str, help='Include a output statistic')
        #parser.add_argument('-e', '--include', required=False, type=str, help='Exclude a output statistic')
        #parser.add_argument('-d', '--defaults', required=False, type=str, help='Use default statistics')
        self.args = parser.parse_args()


    def run_command(self, command):
        try:
            result = subprocess.run(command.to_string(), shell=True, capture_output=True, text=True)
            print(colored(f"Command: {command.to_string()}", 'magenta'))
            print(colored(f"Return code: {result.returncode}", 'magenta'))
            print(colored("STDOUT:", 'green'))
            print(result.stdout)
            if result.stderr:
                print(colored("STDERR:", 'red'))
                print(result.stderr)
        except Exception as e:
            print(colored(f"An error occurred while running command {command.to_string()}. Panic!", 'red'))
            print(colored("Error: " + str(e), 'red'))
            exit(1)
        

    def compress_from_png(self, input_dir, output_dir):
        dataset_name = self.get_dataset_name(input_dir)
        table = Table(f"Compression Data for Image Dataset {dataset_name}",
                ImageCompressionData.get_col_names())       
        input_files = self.get_images_in_dir(input_dir, SupportedImageExt.PNG)

        for png_file in input_files:
            file_name = os.path.split(os.path.splitext(png_file)[0])[1]
            output_file = f"{output_dir}/{file_name}.jxl"

            effort = 1
            while effort <= EFFORT_LEVELS:
                self.run_command(Command(f"cjxl -e {effort} {png_file} {output_file}")) 
                compression_data = ImageCompressionData(
                        input_image_path = png_file,
                        compressed_image_path = output_file,
                        compression_effort = effort)
                table.create_row_from_compression_data(compression_data)
                effort += 1

        return table


    def decompress_to_png(self, input_dir, output_dir):
        run_output_files = []
        
        input_files = self.get_images_in_dir(input_dir, SupportedImageExt.JXL)

        for jxl_file in input_files:
            file_name = os.path.split(os.path.splitext(jxl_file)[0])[1]
            output_file = f"{output_dir}/{file_name}.png"
            self.run_command(Command(f"djxl {jxl_file} {output_file}")) 
            run_output_files.append(output_file)

        return run_output_files


    def compare_images(self, before_dir, after_dir, result_dir):
        before_images = self.get_images_in_dir(before_dir, SupportedImageExt.PNG)
        after_images = self.get_images_in_dir(after_dir, SupportedImageExt.PNG)

        if len(before_images) != len(after_images):
            raise Exception("Mismatch image set lengths.")

        assert os.path.isdir(result_dir), "Result directory does not exist."
    
        i = 0
        while i < len(before_images):
            image_name = os.path.splitext(os.path.split(before_images[i])[1])[0]

            if image_name != os.path.splitext(os.path.split(after_images[i])[1])[0]:
                raise Exception("Mismatch image names.")

            self.run_command(Command(f"{DIFF_TOOL} {before_images[i]} {after_images[i]} {result_dir}/{image_name}-diff.png"))
            i += 1
    

    def main(self):
        self.parse_arguments()
        if self.args.test_set is None:
            datasets = []
        else:
            datasets = [ self.args.test_set ]

        if self.args.test_sets is not None:
            datasets = datasets + self.args.test_sets

        assert len(datasets) > 0, "No datasets provided, using -t, or -ts to provide test dataset, see --help for more."
        self.start_test_run(datasets)


if __name__ == '__main__':
    tester = JXLTester()
    tester.main()
