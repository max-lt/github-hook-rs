use std::io::Read;

use rand::RngCore;

fn stream<R: Sized + Read>(id: &str, name: &str, reader: R) {
    let mut reader = std::io::BufReader::new(reader);
    let mut line = String::new();
    while std::io::BufRead::read_line(&mut reader, &mut line).unwrap() > 0 {
        log::info!("Job {id}:{name}: {}", line.trim());
        line.clear();
    }
}

pub fn run_script(script: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
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
          let job_id = job_id.clone();
          let stdout = child.stdout.take().unwrap();
          std::thread::spawn(move || stream(&job_id, "stdout", stdout))
        };

        let err_h = {
          let job_id = job_id.clone();
          let stderr = child.stderr.take().unwrap();
          std::thread::spawn(move || stream(&job_id, "stderr", stderr))
        };

        // wait out_h and err_h to finish
        out_h.join().unwrap();
        err_h.join().unwrap();

        log::info!("Job {job_id} finished");
    })
}
