use clap::{
    Parser,
    Subcommand,
};
use serde_json;
use serde::{
    Deserialize, Serialize
};
use rand::{thread_rng, Rng};
use actix_web::{
    HttpServer,
    App,
    dev::{
        ServiceRequest, 
        ServiceResponse, 
        fn_service
    },
};
use actix_files::{
    Files,
    NamedFile,
};
use std::{
    env::current_exe,
    net::Ipv4Addr,
    path::{Path,PathBuf},
    fs,
};
use local_ip_address::local_ip;
mod launch;
use launch::launch_browser;


#[derive(Serialize, Deserialize)]
pub struct Configs {
    server_name:String,
    location:String,
    listen:u16
}

#[derive(Parser)]
#[command(author="Imrany <imranmat254@gmail.com>", version, about="Packer is a simple web server used to serve static contents.", long_about = None)]
struct Args {
    /// Path to the folder you want to serve
    #[arg(short, long, value_name= "PATH")]
    root: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Serves a specific folder.
    Serve {
        path: Option<String>,
    },
}

#[actix_web::main]
async fn main(){
    let args = Args::parse();
    let port:u16=thread_rng().gen_range(3000..=8080);

    // let current_exe_path=PathBuf::from(current_exe().unwrap());
    // let configurations_json_file=Path::new(current_exe_path.parent().unwrap()).join("config.json");
    let configurations_json_file=PathBuf::from("config.json");
    let configurations:Configs=serde_json::from_str(fs::read_to_string(configurations_json_file).unwrap().as_str()).unwrap(); 

    if let Some(path) = args.root.as_deref() {
        serve(path.to_string(),port).await;
    }
    
    match &args.command {
        Some(Commands::Serve { path }) => {
            if let Some(path) = path.as_deref() {
                serve(path.to_string(),port).await;
            }else {
                println!("  ERROR Specify a path to serve.");
                println!(" {}",format!(" HINT: To serve the current folder - 'anvel serve ./'."));
            }

        }
        None => {
            serve(configurations.location,configurations.listen).await;
        }
    }
}

async fn serve(path: String, port:u16) {
    let ipv4: (Ipv4Addr, u16)=("0.0.0.0".parse().unwrap(),port);
    let server=HttpServer::new(move ||
        App::new()
            .service(Files::new("/", path.clone()).show_files_listing().index_file("index.html")
                .default_handler(fn_service(|req: ServiceRequest| async {
                    let (req, _) = req.into_parts();
                    let current_exe_path=PathBuf::from(current_exe().unwrap());
                    let file = NamedFile::open_async(Path::new(current_exe_path.parent().unwrap()).join("assets/error_page.html")).await?;
                    let res = file.into_response(&req);
                    Ok(ServiceResponse::new(req, res))
                }))
            )
    )
    .bind(ipv4);
    match server {
        Ok(serve) => {
            let url;
            match local_ip(){
                Ok(ip) => {
                    url=format!("http://{ip}:{port}");
                    println!(" Networkhost: {ip} ");
                    println!(" Url: {url} ");
                },
                Err(_) => {
                    url=format!("http://localhost:{port}");
                    println!(" Localhost: localhost ");
                    println!(" Url: {url}");
                }
            };
            if let Err(e) = launch_browser(&url).await {
                println!(" ERROR An error occurred when opening {url} {e}");
            };
            serve.run().await.unwrap_or_else(|err| println!(" ERROR {} ",err));
        },
        Err(e) =>  println!(" {} ",e)
    }
}
