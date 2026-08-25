use actix_files::Files;
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    middleware::Logger,
    App, HttpResponse, HttpServer,
};
use clap::{Parser, Subcommand};
use env_logger::Env;
use local_ip_address::local_ip;
use log::{error, info, warn};
use rand::{thread_rng, Rng};
use self_update::cargo_crate_version;
use serde::{Deserialize, Serialize};
use std::{
    env::{current_dir, current_exe},
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

mod launch;
use launch::launch_browser;
mod error_page;
use error_page::get_error_page_html;
mod tunnel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configs {
    pub server_name: String,
    pub location: PathBuf,
    pub listen: u16,
}

#[derive(Parser)]
#[command(
    author = "Imrany <imranmat254@gmail.com>",
    version,
    about = "Packer is a simple web server used to serve static contents.",
    long_about = None
)]
struct Args {
    /// Specify the port you want to serve on
    #[arg(short, long, value_name = "PORT", global = true)]
    port: Option<u16>,

    /// Expose the running server to the public internet via a free Cloudflare Tunnel
    #[arg(short = 't', long, global = true)]
    tunnel: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Serves a specific folder.
    Serve {
        /// The path to serve (e.g., ./)
        path: String,
    },
    /// Updates the packer binary to the latest version.
    Update,

    /// Uninstalls the packer binary.
    Uninstall,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger with default log level set to "info"
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let args = Args::parse();

    // 1. SAFE PATH RESOLUTION
    let exe_path = current_exe().unwrap_or_else(|_| PathBuf::from("./packer"));
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
    let config_file_path = exe_dir.join("config.json");

    // Default to a random port if one wasn't passed in the root CLI args
    let default_port: u16 = args
        .port
        .unwrap_or_else(|| thread_rng().gen_range(3000..=8080));

    // 2. CONFIGURATION I/O
    let config_content = match fs::read_to_string(&config_file_path) {
        Ok(content) => content,
        Err(_) => {
            info!("Creating config.json at {:?}...", config_file_path);

            let default_config = Configs {
                server_name: String::from("My web server"),
                location: current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                listen: default_port,
            };

            let json = serde_json::to_string_pretty(&default_config)
                .unwrap_or_else(|_| "{}".to_string());

            if let Err(e) = fs::write(&config_file_path, &json) {
                warn!("Could not save config.json: {}", e);
            }
            json
        }
    };

    // Parse the JSON safely
    let configurations: Configs = serde_json::from_str(&config_content).unwrap_or_else(|_| {
        warn!("Failed to parse config.json. Using fallback settings.");
        Configs {
            server_name: String::from("Fallback Server"),
            location: current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            listen: default_port,
        }
    });

    // 3. CLI COMMAND EXECUTION
    match &args.command {
        Some(Commands::Serve { path }) => {
            let serve_path = PathBuf::from(path);
            let effective_port = args.port.unwrap_or(configurations.listen);
            serve(serve_path, effective_port, exe_dir, None, args.tunnel).await?;
        }
        Some(Commands::Update) => {
            if let Err(e) = update_packer() {
                error!("Failed to update: {}", e);
            }
        }
        Some(Commands::Uninstall) => {
            if let Err(e) = uninstall_packer() {
                error!("Failed to uninstall: {}", e);
            }
        }
        None => {
            let effective_port = args.port.unwrap_or(configurations.listen);
            serve(
                configurations.location.clone(),
                effective_port,
                exe_dir,
                Some(&configurations),
                args.tunnel,
            )
            .await?;
        }
    }

    Ok(())
}

async fn serve(
    path: PathBuf,
    port: u16,
    exe_dir: &Path,
    configurations: Option<&Configs>,
    enable_tunnel: bool,
) -> std::io::Result<()> {
    let ipv4: (Ipv4Addr, u16) = ("0.0.0.0".parse().unwrap(), port);

    if !path.exists() {
        warn!("The path {:?} does not exist. Serving may fail.", path);
    }

    let config_file_path = exe_dir.join("config.json");
    let config_display_path = config_file_path.to_string_lossy().to_string();

    let server = HttpServer::new(move || {
        let err_path = config_display_path.clone();
        App::new()
            // Enable HTTP request logging middleware
            .wrap(Logger::default())
            .service(
                Files::new("/", &path)
                    .show_files_listing()
                    .index_file("index.html")
                    // Directly serve the embedded HTML string for all 404 handler routes
                    .default_handler(fn_service(move |req: ServiceRequest| {
                        let path_for_404 = err_path.clone();
                        async move {
                            let (req, _) = req.into_parts();
                            let body_html = get_error_page_html(&path_for_404);

                            let res = HttpResponse::NotFound()
                                .content_type("text/html; charset=utf-8")
                                .body(body_html);

                            Ok(ServiceResponse::new(req, res))
                        }
                    })),
            )
    })
    .bind(ipv4)?;

    let (url, localhost_url) = match local_ip() {
        Ok(ip) => (
            format!("http://{}:{}", ip, port),
            format!("http://localhost:{}", port),
        ),
        Err(_) => (
            format!("http://localhost:{}", port),
            format!("http://localhost:{}", port),
        ),
    };

    if let Some(configs) = configurations {
        info!("Server name: {}", configs.server_name);
    }

    info!("Network host: {}", url);
    info!("Localhost:    {}", localhost_url);

    // 4. OPTIONAL PUBLIC TUNNEL (Cloudflare quick tunnel, free, no account needed)
    let _tunnel_guard = if enable_tunnel {
        match tunnel::ensure_cloudflared(exe_dir) {
            Ok(cloudflared_bin) => {
                info!("Starting Cloudflare quick tunnel via {:?}...", cloudflared_bin);
                match tunnel::start_cloudflare_tunnel(&cloudflared_bin, port) {
                    Ok(t) => {
                        info!("🌐 Public URL: {}", t.public_url);
                        info!("   (This is a free, temporary trycloudflare.com URL. It stays online only while packer is running.)");
                        Some(t)
                    }
                    Err(e) => {
                        error!("Failed to start public tunnel: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Could not prepare cloudflared binary: {}", e);
                tunnel::print_install_instructions();
                None
            }
        }
    } else {
        None
    };

    if let Err(e) = launch_browser(&localhost_url).await {
        warn!("Could not automatically open browser: {}", e);
    }

    server.run().await
}

fn update_packer() -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking GitHub for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("imrany")
        .repo_name(env!("CARGO_PKG_NAME"))
        .bin_name(env!("CARGO_PKG_NAME"))
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    match status {
        self_update::Status::UpToDate(v) => {
            info!("✨ Packer is already up to date! (v{})", v);
        }
        self_update::Status::Updated(v) => {
            info!("✅ Successfully updated Packer to v{}!", v);
        }
    }

    Ok(())
}

fn uninstall_packer() -> Result<(), Box<dyn std::error::Error>> {
    info!("Uninstalling Packer...");
    let bin_path = PathBuf::from("/usr/local/bin");
    let binary_path = bin_path.join(env!("CARGO_PKG_NAME"));
    std::process::Command::new("rm")
        .arg(&binary_path)
        .status()
        .map_err(|e| Box::new(e))?;
    info!("Packer uninstalled successfully.");
    Ok(())
}
