#[macro_use] extern crate rocket;
use rocket::fs::TempFile;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use serde::{Serialize, Deserialize};
use tera::{Context, Tera};

#[derive(FromForm)]
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


#[post("/upload", data = "<form>")]
async fn upload(mut form: Form<Upload<'_>>) -> std::io::Result<()> {
    //let filename: String = form.filename.clone();
    let filename = "foo.txt";
    form.myfile.persist_to(filename).await?;
    Ok(())
}

#[get("/")]
fn root() -> RawHtml<String> {
    let mut tera = Tera::default();
    tera.add_template_file("./index.html", Some("index.html")).unwrap();
    let mut context = Context::new();
    context.insert("breadcrumbs", &vec![
                   Breadcrumb { name: "foo".to_string(), path: "/no/where/".to_string() }

    ]);
    context.insert("files", &vec![LocalFile{ name: "foo".to_string(), path: "/home/tristan".to_string(), path_type: "File".to_string()}]);
    context.insert("dir_name", "~");

    RawHtml(tera.render("index.html", &context).unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![root])
        .mount("/", routes![upload])
}
