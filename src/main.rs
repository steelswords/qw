#[macro_use] extern crate rocket;
use rocket::fs::TempFile;
use rocket::form::Form;

#[derive(FromForm)]
struct Upload<'f> {
    myfile: TempFile<'f>,
}

#[post("/upload", data = "<form>")]
async fn upload(mut form: Form<Upload<'_>>) -> std::io::Result<()> {
    form.myfile.persist_to("/tmp/complete/file.txt").await?;
    Ok(())
}

#[get("/")]
fn root() -> &'static str {
    "Hello, there"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![root])
        .mount("/", routes![upload])
}
