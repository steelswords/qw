#[macro_use] extern crate rocket;
use std::path::PathBuf;
use std::fs;

use rocket::fs::NamedFile;
use rocket::http::uri::Path;
use rocket::{fs::TempFile, figment::providers::Format};
use rocket::form::Form;
use rocket::response::Responder;
use rocket::response::content::RawHtml;
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
    NamedFile(NamedFile),
}

fn get_all_files_in_directory(parent_path: &PathBuf) -> Vec<LocalFile> {
    println!("parent_path = {:?}", parent_path);
    let paths = fs::read_dir(parent_path).unwrap_or(fs::read_dir(PathBuf::from(".")).unwrap());
    let mut return_vec = vec![];

    for path in paths {
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
            let download_path = path.path();

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
        form.myfile.persist_to(file).await?;
    }
    Ok(())
}

#[get("/<path..>")]
async fn root(path: PathBuf) -> FileOrIndexResponse {
    if path.is_dir() {
        let mut tera = Tera::default();
        // TODO: Make this all internal
        tera.add_template_file("./index.html", Some("index.html")).unwrap();

        let mut context = Context::new();
        // TODO
        context.insert("breadcrumbs", &vec![
                       Breadcrumb { name: "foo".to_string(), path: "/no/where/".to_string() }

        ]);
        // TODO
        //context.insert("files", &vec![LocalFile{ name: "foo".to_string(), path: "/home/tristan".to_string(), path_type: "File".to_string()}]);
        context.insert("files", &get_all_files_in_directory(&path));
        context.insert("dir_name", path.as_os_str());

        FileOrIndexResponse::Html(RawHtml(tera.render("index.html", &context).unwrap()))
    }
    else {
        FileOrIndexResponse::NamedFile(NamedFile::open(PathBuf::from(&path)).await.expect(format!("Could not serve file {:?}", path.into_os_string()).as_str()))
    }
}

#[launch]
fn rocket() -> _ {

    rocket::build()
        .mount("/", routes![root])
        .mount("/", routes![upload])
}
