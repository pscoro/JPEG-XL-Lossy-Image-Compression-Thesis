use std::collections::HashMap;
use std::error::Error;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DockerManager {
    pub id: usize,
    pub dockerfile: String,
    pub image_name: Option<String>,
    pub container_name: Option<String>,
    containers: HashMap<usize, String>,
}

impl DockerManager {
    pub const IMAGE_NAME: &'static str = "benchmark-libjxl-image";
    pub const CONTAINER_NAME: &'static str = "benchmark-libjxl-container";

    pub fn new(dockerfile: &str, id: usize) -> DockerManager {
        DockerManager {
            id,
            dockerfile: String::from(dockerfile),
            image_name: Some(String::from(DockerManager::IMAGE_NAME)),
            container_name: Some(format!("{}-{}", String::from(DockerManager::CONTAINER_NAME), id)),
            containers: HashMap::new(),
        }
    }

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

    pub fn execute_cjxl(
        &self,
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

        self.execute_in_container("../libjxl/build/tools/cjxl", args)
    }

    /// Sets up a docker container for a benchmark worker.
    /// # Returns
    /// * `Result<(), Error>` - An error if the setup fails.
    pub fn setup(&mut self, worker_id: usize) -> Result<(), Box<dyn Error>> {
        println!("Setting up docker container...");

        // Create the container.
        println!("Creating docker container...");
        self.execute_command(
            Command::new("docker")
                .arg("build")
                .arg("-t")
                .arg(format!("ubuntu:{}", self.image_name.as_ref().unwrap()))
                .arg("-f")
                .arg(self.dockerfile.as_str())
                .arg("."),
        )?;

        // Check if container is running.
        /*        // If it is, stop it.
                println!("Checking if docker container is running...");
                let output = self.execute_command(
                    Command::new("docker")
                        .arg("ps")
                        .arg("--filter")
                        .arg(format!("name={}", self.container_name.as_ref().unwrap()))
                        .arg("--format")
                        .arg("{{.Names}}")
                )?;
                if output.len() > 0 {
                    println!("Stopping docker container...");
                    self.execute_command(
                        Command::new("docker")
                            .arg("stop")
                            .arg(self.container_name.as_ref().unwrap())
                    )?;
                }
        */
        let worker_container_name = self.container_name.as_ref().unwrap();
        self.containers
            .insert(worker_id, worker_container_name.clone());
        // Start the container.
        println!("Starting docker container...");
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
    /// * `command` - The command to execute in the docker container.
    /// * `args` - The arguments to pass to the command.
    ///
    /// # Returns
    /// * `Result<Result<String, String>, Error>` - The result of the command.
    pub fn execute_in_container(
        &self,
        subcommand: &str,
        args: Vec<&str>,
    ) -> Result<Result<String, String>, Box<dyn Error>> {
        println!("Executing command: {} {}", subcommand, args.join(" "));

        let mut command = Command::new("docker");
        command.arg("exec");
        command.arg(self.container_name.as_ref().unwrap());
        command.arg(subcommand);
        command.args(args.as_slice());

        let output = command.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(Ok(stdout))
        } else {
            Ok(Err(stderr))
        }
    }

    /// Tears down the docker container.
    ///
    /// # Returns
    /// * `Result<(), Error>` - An error if the teardown fails.
    pub fn teardown(&self) -> Result<(), Box<dyn Error>> {
        println!("Tearing down docker container...");
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
        println!("Stopping docker container...");
        self.execute_command(
            Command::new("docker")
                .arg("stop")
                .arg(self.container_name.as_ref().unwrap()),
        )?;

        // Remove the container.
        println!("Removing docker container...");
        self.execute_command(
            Command::new("docker")
                .arg("rm")
                .arg(self.container_name.as_ref().unwrap()),
        )?;

        // Remove the image.
        println!("Removing docker image...");
        self.execute_command(
            Command::new("docker")
                .arg("rmi")
                .arg(format!("ubuntu:{}", self.image_name.as_ref().unwrap())),
        )?;

        Ok(())
    }
}
