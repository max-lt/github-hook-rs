use actix_web::web::Bytes;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use sha2::digest::Mac;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Project {
    pub secret: String,
    pub script: String,
    pub branch: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    pub repositories: std::collections::BTreeMap<String, Project>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SIG_HEADER: &str = "x-hub-signature-256";

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not Found")
}

#[actix_web::get("/github-hook/version")]
async fn get_version() -> &'static str {
    VERSION
}

#[actix_web::post("/github-hook/{project_id}")]
async fn github_hook(
    config: Data<Config>,
    project_id: Path<String>,
    req: HttpRequest,
    data: Bytes,
) -> impl Responder {
    let project_id = project_id.into_inner();

    let config = match config.repositories.get(&project_id) {
        Some(c) => c,
        None => {
            return HttpResponse::NotFound().body("Invalid project_id");
        }
    };

    log::debug!("Found config for project \"{project_id}\": {:#?}", config);

    // Verify signature
    {
        let gh_sig = match req.headers().get(SIG_HEADER) {
            Some(h) => h.to_str().unwrap(),
            None => {
                return HttpResponse::BadRequest().body("Missing x-hub-signature");
            }
        };

        log::debug!("Received a GitHub hook for project: \"{project_id}\" with signature {gh_sig}");

        let hash: String = {
            let mut mac = HmacSha256::new_from_slice(config.secret.as_bytes()).unwrap();
            mac.update(data.as_ref());
            hex::encode(mac.finalize().into_bytes())
        };

        log::debug!("Computed hash: {hash}");

        if gh_sig != format!("sha256={}", hash) {
            return HttpResponse::BadRequest().body("Invalid x-hub-signature");
        }
    }

    let gh_event = match req.headers().get("x-github-event") {
        Some(h) => h.to_str().unwrap(),
        None => {
            return HttpResponse::BadRequest().body("Missing x-github-event");
        }
    };

    // Only handle push events
    if gh_event != "push" {
        return HttpResponse::Ok().body("ok");
    }

    // Get JSON body
    let json: serde_json::Value = serde_json::from_slice(data.as_ref()).unwrap();
    log::trace!("JSON: {}", serde_json::to_string_pretty(&json).unwrap());

    /*
     * Note that the event body may vary depending on the event source (github builtin hooks / github actions hooks)
     * Github builtin hooks: https://docs.github.com/en/developers/webhooks-and-events/webhooks/webhook-events-and-payloads#push
     * Github actions hooks: { ref, event, repository, commit, head, workflow, requestID } (strings only)
     */
    let reference = json.get("ref").unwrap().as_str().unwrap();
    log::trace!("Ref: {:?}", reference);

    let branch = reference.split('/').last().unwrap();
    log::debug!("Branch: {:?}", branch);

    if let Some(ref expected_branch) = config.branch {
        if branch != expected_branch {
            return HttpResponse::Ok().body("ok");
        }
    }

    run_script(config.script.clone());

    HttpResponse::Ok().body("ok")
}

fn run_script(script: String) {
    log::info!("Running script: {}", script);

    std::thread::spawn(move || {
        match subprocess::Exec::shell(script).capture() {
            Ok(out) => {
                log::info!("Script output: {}", out.stdout_str());
            }
            Err(e) => {
                log::error!("Error running script: {}", e);
            }
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !std::env::var("RUST_LOG").is_ok() {
        std::env::set_var("RUST_LOG", "github_hook_rs=debug");
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port.parse::<u16>().expect("PORT must be a number");

    env_logger::init();

    let config_path = std::env::var("CONFIG").expect("Environment variable CONFIG is required");
    let config = std::fs::File::open(&config_path)
        .expect(format!("Could not open config file: {}", &config_path).as_str());
    let config: Config = serde_yaml::from_reader(config)
        .expect(format!("Could not parse config file: {}", &config_path).as_str());

    log::debug!("Config: {:#?}", config);

    log::info!("Starting server on port {}", port);

    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(Data::new(config.clone()))
            .service(get_version)
            .service(github_hook)
            .default_service(actix_web::web::to(not_found))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
