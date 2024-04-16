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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configs {
    server_name:String,
    location:PathBuf,
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

    let current_exe_path=PathBuf::from(current_exe().unwrap());
    let configurations_json_file_path=Path::new(current_exe_path.parent().unwrap().parent().unwrap()).join("config.json");
    // let configurations_json_file_path=PathBuf::from("config.json");
    let read_json_content=match fs::read_to_string(&configurations_json_file_path){
        Ok(v)=>v,
        Err(_)=>{
            println!(" Creating config.json...\n ");
            let data=Configs{
                server_name:String::from("My web server"),
                location:PathBuf::from(current_exe_path.parent().unwrap().parent().unwrap()).join("error"),
                listen:80
            };
            let data_as_json=serde_json::to_string(&data).unwrap();
            fs::File::create(&configurations_json_file_path).unwrap();
            fs::write(configurations_json_file_path,&data_as_json).unwrap();
            data_as_json
        }
    };
    let configurations:Configs=serde_json::from_str(read_json_content.as_str()).unwrap(); 

    if let Some(path) = args.root.as_deref() {
        serve(path.into(),port, None).await;
    }
    
    match &args.command {
        Some(Commands::Serve { path }) => {
            if let Some(path) = path.as_deref() {
                serve(path.into(),port, None).await;
            }else {
                println!("  ERROR Specify a path to serve.");
                println!(" {}",format!(" HINT: To serve the current folder - 'packer serve ./'."));
            }

        }
        None => {
            serve(configurations.clone().location,configurations.listen, Some(&configurations)).await;
        }
    }
}

async fn serve(path: PathBuf, port:u16, configurations:Option<&Configs>) {
    let ipv4: (Ipv4Addr, u16)=("0.0.0.0".parse().unwrap(),port);
    let server=HttpServer::new(move ||
        App::new()
            .service(Files::new("/", &path).show_files_listing().index_file("index.html")
                .default_handler(fn_service(|req: ServiceRequest| async {
                    let (req, _) = req.into_parts();
                    let current_exe_path=PathBuf::from(current_exe().unwrap());
                    let file = NamedFile::open_async(Path::new(current_exe_path.parent().unwrap().parent().unwrap()).join("error/index.html")).await?;
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
                    if let Some(configs) = configurations {
                        println!(" Server name: {} ", configs.server_name);
                    };
                    println!(" Networkhost: {ip} ");
                    println!(" Url: {url} ");
                },
                Err(_) => {
                    url=format!("http://localhost:{port}");
                    if let Some(configs) = configurations {
                        println!(" Server name: {} ", configs.server_name);
                    };
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
