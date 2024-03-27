use std::collections::HashMap;
use std::error::Error;
use std::process::Command;

/// A struct that manages the docker container for a benchmark worker.
#[derive(Debug, Clone)]
pub struct DockerManager {
    pub id: usize,
    pub dockerfile: String,
    pub image_name: Option<String>,
    pub container_name: Option<String>,
    containers: HashMap<usize, String>,
}

impl DockerManager {
    /// The name of the docker image and the base name of the docker container.
    pub const IMAGE_NAME: &'static str = "benchmark-libjxl-image";
    pub const CONTAINER_NAME: &'static str = "benchmark-libjxl-container";

    /// Creates a new Docker manager instance.
    ///
    /// # Arguments
    /// * `dockerfile` - The path to the Dockerfile to use for the container.
    /// * `id` - The ID of the worker.
    ///
    /// # Returns
    /// * `DockerManager` - The new Docker manager instance.
    pub fn new(dockerfile: &str, id: usize) -> DockerManager {
        // The id is appended to the container name to ensure uniqueness.
        DockerManager {
            id,
            dockerfile: String::from(dockerfile),
            image_name: Some(String::from(DockerManager::IMAGE_NAME)),
            container_name: Some(format!(
                "{}-{}",
                String::from(DockerManager::CONTAINER_NAME),
                id
            )),
            containers: HashMap::new(),
        }
    }

    /// Executes the given command on the given local machine and returns the output.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The stdout of the command or an error with the stderr if the 
    /// command fails.
    fn execute_command(&self, command: &mut Command) -> Result<String, Box<dyn Error>> {
        let output = command
            .output()
            .expect(format!("failed to execute command: {:?}", command).as_str());

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).parse()?)
        } else {
            Err(Box::from(String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Copies a file from the docker container to the local machine.
    ///
    /// # Arguments
    /// * `file_path` - The path to the file in the docker container.
    /// * `dest_path` - The path to copy the file to on the local machine.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The stdout of the command or an error with the stderr if the 
    /// command fails.
    pub fn retrieve_file(
        &self,
        file_path: String,
        dest_path: String,
    ) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("cp");
        command.arg(format!(
            "{}:{}",
            self.container_name.as_ref().unwrap(),
            file_path
        ));
        command.arg(dest_path);

        self.execute_command(&mut command)
    }

    /// Executes the cjxl encoding tool in the docker container.
    ///
    /// # Arguments
    /// * `input_file` - The path to the input image file to encode.
    /// * `output_file` - The name of the output file to create (in the docker container).
    /// * `distance` - The cjxl Butteraugli distance (quality) to use for the encoding.
    /// * `effort` - The cjxl effort level to use for the encoding.
    ///
    /// # Returns
    /// * `Result<Result<String, String>, Error>` - The result of the command as a (stdout, stderr)
    /// tuple or an error if there was an issue executing the command.
    pub fn execute_cjxl(
        &self,
        input_file: String,
        output_file: String,
        distance: f64,
        effort: u32,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        // Create the output directory if it doesn't exist.
        _ = self.execute_in_container(
            "mkdir",
            vec![
                "-p",
                format!(
                    "{}",
                    output_file
                        .clone()
                        .split("/")
                        .take(output_file.split("/").count() - 1)
                        .collect::<Vec<&str>>()
                        .join("/")
                )
                .as_str(),
            ],
        )?;
        
        // Add the distance and effort flags to the command.
        let distance = format!("--distance={}", distance);
        let effort = format!("--effort={}", effort);
        let args = vec![
            input_file.as_str(),
            output_file.as_str(),
            distance.as_str(),
            effort.as_str(),
        ];

        // Execute the cjxl command in the docker container.
        self.execute_in_container("/libjxl/build/tools/cjxl", args)
    }

    /// Executes the JPEG XL SSIMULACRA2 benchmarking tool in the docker container.
    ///
    /// # Arguments
    /// * `orig_file` - The path to the original image file.
    /// * `comp_file` - The path to the compressed image file.
    ///
    /// # Returns
    /// * `Result<Result<String, String>, Error>` - The result of the command as a (stdout, stderr)
    /// tuple or an error if there was an issue executing the command.
    pub fn execute_ssimulacra2(
        &self,
        orig_file: String,
        comp_file: String,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        let args = vec![orig_file.as_str(), comp_file.as_str()];

        self.execute_in_container("../libjxl/build/tools/ssimulacra2", args)
    }

    /// Executes the libjxl Butteraugli benchmarking tool in the docker container.
    ///
    /// # Arguments
    /// * `orig_file` - The path to the original image file.
    /// * `comp_file` - The path to the compressed image file.
    ///
    /// # Returns
    /// * `Result<Result<String, String>, Error>` - The result of the command as a (stdout, stderr)
    /// tuple or an error if there was an issue executing the command.
    pub fn execute_butteraugli(
        &self,
        orig_file: String,
        comp_file: String,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        let args = vec![orig_file.as_str(), comp_file.as_str()];

        self.execute_in_container("/libjxl/build/tools/butteraugli_main", args)
    }

    /// Sets up a docker container for a benchmark worker.
    ///
    /// # Arguments
    /// * `worker_id` - The ID of the worker.
    ///
    /// # Returns
    /// * `Result<(), Error>` - An error if the setup fails.
    pub fn setup(&mut self, worker_id: usize) -> Result<(), Box<dyn Error>> {
        // Build the docker image.
        match self.execute_command(
            Command::new("docker")
                .arg("build")
                .arg("-t")
                .arg(format!("ubuntu:{}", self.image_name.as_ref().unwrap()))
                .arg("-f")
                .arg(self.dockerfile.as_str())
                .arg("."),
        ) {
            Ok(_) => {}
            Err(_) => {
                return Err(Box::from("Failed to build docker image"));
            }
        }

        let worker_container_name = self.container_name.as_ref().unwrap();
        self.containers
            .insert(worker_id, worker_container_name.clone());

        // Start the container.
        self.execute_command(
            Command::new("docker")
                .arg("run")
                .arg("--name")
                .arg(worker_container_name)
                .arg("-dit")
                .arg(format!("ubuntu:{}", self.image_name.as_ref().unwrap())),
        )?;

        Ok(())
    }

    /// Executes the given command in the docker container.
    ///
    /// # Arguments
    /// * `subcommand` - The subcommand to execute with `docker exec`.
    /// * `args` - The arguments to pass to the command.
    ///
    /// # Returns
    /// * `Result<Result<String, String>, Error>` - The result of the command as a (stdout, stderr)
    /// tuple or an error if there was an issue executing the command.
    pub fn execute_in_container(
        &self,
        subcommand: &str,
        args: Vec<&str>,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg("-w");
        command.arg("/temp");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg(subcommand);
        command.args(args.as_slice());

        let output = command.output()?;

        // Convert the output to a string.
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(Ok(stdout))
        } else {
            if stderr.len() > 0 {
                Ok(Err(stderr))
            } else {
                Ok(Err(stdout))
            }
        }
    }

    /// Tears down the docker container.
    ///
    /// # Returns
    /// * `Result<(), Error>` - An error if the teardown fails.
    pub fn teardown(&self) -> Result<(), Box<dyn Error>> {
        // clean the /temp folder
        self.execute_command(
            Command::new("docker")
                .arg("exec")
                .arg(self.container_name.as_ref().unwrap())
                .arg("rm")
                .arg("-rf")
                .arg("/temp/*"),
        )?;

        // Stop the container.
        self.execute_command(
            Command::new("docker")
                .arg("stop")
                .arg(self.container_name.as_ref().unwrap()),
        )?;

        // Remove the container.
        self.execute_command(
            Command::new("docker")
                .arg("rm")
                .arg(self.container_name.as_ref().unwrap()),
        )?;

        // Remove the image.
        self.execute_command(
            Command::new("docker")
                .arg("rmi")
                .arg(format!("ubuntu:{}", self.image_name.as_ref().unwrap())),
        )?;

        Ok(())
    }

    /// Changes the libjxl commit in the docker container.
    ///
    /// # Arguments
    /// * `commit` - The commit hash or branch name to checkout in the libjxl repository.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The output of the command or an error if the command fails.
    pub fn change_libjxl_commit(&self, commit: &str) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg("bash");
        command.arg("-c");
        command.arg(format!(
            "cd /libjxl && git fetch origin && git checkout {} && cd -",
            commit
        ));

        self.execute_command(&mut command)
    }

    /// Applies a local git diff to the libjxl repository in the docker container.
    ///
    /// # Arguments
    /// * `diff` - The diff to apply to the libjxl repository.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The output of the command or an error if the command fails.
    pub fn apply_diff(&self, diff: &str) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg("bash");
        command.arg("-c");
        command.arg(format!("cd /libjxl && git apply {} && cd -", diff));

        self.execute_command(&mut command)
    }

    /// Applies libjxl changes from the local machine to the libjxl repository in the docker
    /// container using a git diff. The local changes are stored in a file called `local.diff` in
    /// the current directory.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The output of the command or an error if the command fails.
    pub fn apply_local_as_diff(&self) -> Result<String, Box<dyn Error>> {
        // Copy diff to docker container
        let _ = self.execute_command(Command::new("docker").arg("cp").arg("local.diff").arg(
            format!(
                "{}:/libjxl/local.diff",
                self.container_name.as_ref().unwrap()
            ),
        ));

        let _ = self.apply_diff("local.diff");
        Ok(String::from("Applied local folder as diff"))
    }

    /// Builds the libjxl library in the docker container.
    /// This should be run after changing the libjxl commit or applying a diff.
    ///
    /// # Returns
    /// * `Result<String, Error>` - The output of the command or an error if the command fails.
    pub fn build_libjxl(&self) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg("bash");
        command.arg("-c");
        command.arg("cd /libjxl && SKIP_TEST=1 ./ci.sh opt; exit 0 && cd -");

        self.execute_command(&mut command)
    }

    /// Cleans the libjxl repository in the docker container.
    /// This should be run before changing the libjxl commit or applying a diff for a clean slate.
    pub fn clean_libjxl(&self) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg("bash");
        command.arg("-c");
        command.arg("cd /libjxl && git clean -fdx && cd -");

        self.execute_command(&mut command)
    }
}
