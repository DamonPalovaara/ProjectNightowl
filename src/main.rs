use rocket_contrib::serve::StaticFiles;

fn main() {
    rocket::ignite()
        .mount("/", StaticFiles::from("./public"))
        .launch();
}
