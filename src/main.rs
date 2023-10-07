#[macro_use] extern crate rocket;
use std::net::{Ipv4Addr, IpAddr};
use std::path::{PathBuf, Ancestors};
use std::fs;

use rocket::data::Limits;
use rocket::data::ToByteUnit;
use rocket::Config;
use rocket::fs::TempFile;
use rocket::form::Form;
use rocket::response::Responder;
use rocket::response::content::RawHtml;
use rocket_download_response::DownloadResponse;
use serde::{Serialize, Deserialize};
use tera::{Context, Tera};

#[derive(FromForm, Debug)]
struct Upload<'f> {
    myfile: TempFile<'f>,
    //filename: String,
}

#[derive(Serialize, Deserialize)]
struct LocalFile {
    pub path_type: String,
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct Breadcrumb {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Responder)]
pub enum FileOrIndexResponse {
    Html(RawHtml<String>),
    DownloadResponse(DownloadResponse),
}

fn get_all_files_in_directory(parent_path: &PathBuf) -> Vec<LocalFile> {
    println!("parent_path = {:?}", parent_path);
    let local_files = fs::read_dir(parent_path).unwrap_or(fs::read_dir(PathBuf::from(".")).unwrap());
    let mut return_vec = vec![];

    for path in local_files {
        if let Ok(path) = path {
            println!("Name: {}", path.path().display());
            // Get the right path type
            let path_type = match path.file_type() {
                Ok(file_type) => {
                    if file_type.is_dir() {
                        "Dir"
                    }
                    else {
                        "File"
                    }
                },
                Err(_) => "File",
            };

            //Construct the right relative path
            let download_path = parent_path.join(path.file_name());

            return_vec.push(
                LocalFile {
                    // TODO
                    path_type: path_type.to_string(),
                    // TODO
                    path: download_path.into_os_string().into_string().unwrap(),
                    name: path.file_name().into_string().expect("Could not stringify file name"),
                }
            );
        }
    }
    return_vec
}

#[post("/<path..>", data = "<form>")]
async fn upload(path: PathBuf, mut form: Form<Upload<'_>>) -> std::io::Result<()> {
    //let filename: String = form.filename.clone();
    println!("Got form data: {:?}", &form);
    println!("Want to write to file {:?}", form.myfile.raw_name());
    if let Some(filename) = form.myfile.raw_name() {
        // TODO: Sanitize this input
        let filename = filename.dangerous_unsafe_unsanitized_raw().as_str();
        let file = path.join(filename);
        println!("Saving file to {:?}", file);
        // `persist_to` actually fails when the temp directory and the current directory are
        // on different logical devices (i.e. different filesystems). We match and do a less-
        // performant move_copy_to instead when there's a failure.
        match form.myfile.persist_to(&file).await {
            Ok(_) => {},
            Err(_) => {
                form.myfile
                    .move_copy_to(&file)
                    .await
                    .expect("Could not persist file.");
            }
        };
    }
    Ok(())
}

#[get("/<path..>")]
async fn root(path: PathBuf) -> Option<FileOrIndexResponse> {
    let current_working_directory = std::env::current_dir()
        .expect("Invalid current working directory. Do you have permissions to access it? Does it still exist?");
    let absolute_path = current_working_directory.join(&path);
    println!("Serving relative path {:?}, Absolute path: {:?}", &path, &absolute_path);

    if absolute_path.is_dir() {
        let mut tera = Tera::default();
        // TODO: Make this all internal
        tera.add_template_file("./index.html", Some("index.html")).unwrap();

        let mut context = Context::new();
        // TODO
        let mut breadcrumb_vec: Vec<Breadcrumb> = vec![];
        for ancestor in absolute_path.ancestors() {
            if let Some(ancestor_without_his_ancestors) = ancestor.file_name() {
                breadcrumb_vec.push(
                    Breadcrumb {
                        name: String::from(ancestor_without_his_ancestors.to_str().unwrap()),
                        path: String::from(ancestor_without_his_ancestors.to_str().unwrap()),
                    }
                );
            }
        }
        let breadcrumb_vec = breadcrumb_vec.iter().rev().collect::<Vec<&Breadcrumb>>();
        context.insert("breadcrumbs", &breadcrumb_vec);
        context.insert("files", &get_all_files_in_directory(&path));
        context.insert("dir_name", path.as_os_str());

        Some(FileOrIndexResponse::Html(RawHtml(tera.render("index.html", &context).unwrap())))
    }
    else {
        if path.as_os_str() == "favicon.ico" {
            return None;
        }
        println!("Sending file {:?}", &path);
        Some(FileOrIndexResponse::DownloadResponse(
            DownloadResponse::from_file(
                PathBuf::from(&path),
                Some(path.file_name().unwrap().to_str().unwrap()),
                None
            ).await
             .expect(format!("Could not serve file {:?}", path.into_os_string()).as_str())
        ))
    }
}

#[launch]
fn rocket() -> _ {

    let config = Config {
        limits: Limits::default()
            .limit("data-form", 100.gibibytes())
            .limit("file", 100.gibibytes()),
        address: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
        ..Config::default()
    };

    rocket::custom(&config)
        .mount("/", routes![root])
        .mount("/", routes![upload])
}
