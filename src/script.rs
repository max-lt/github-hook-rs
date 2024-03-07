use std::io::Read;
use std::io::BufRead;
use std::thread::spawn;
use std::thread::JoinHandle;
use rand::RngCore;

fn stream<R: Read + Send + 'static>(
    id: String,
    name: &'static str,
    reader: R,
) -> JoinHandle<()> {
    spawn(move || {
        let lines = std::io::BufReader::new(reader).lines();

        for line in lines {
            log::info!("Job {id}:{name}: {}", line.unwrap());
        }
    })
}

pub fn run_script(script: String) -> JoinHandle<()> {
    spawn(move || {
        let job_id = {
            let mut data = [0u8; 4];
            rand::thread_rng().fill_bytes(&mut data);
            hex::encode(data)
        };

        log::info!("Job {job_id} running script {script}");

        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start script");

        let out_h = {
            let stdout = child.stdout.take().unwrap();
            stream(job_id.to_string(), "stdout", stdout)
        };

        let err_h = {
            let stderr = child.stderr.take().unwrap();
            stream(job_id.to_string(), "stderr", stderr)
        };

        // wait out_h and err_h to finish
        out_h.join().unwrap();
        err_h.join().unwrap();

        log::info!("Job {job_id} finished");
    })
}
