#[macro_use] extern crate rocket;

use rocket::tokio::time::{Duration, interval};
use rocket::response::stream::TextStream;

#[get("/")]
fn index() -> &'static str { 
    "Hello, world!"
}

#[post("/generate")]
fn generate() -> TextStream![&'static str] {
    TextStream! {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            yield "hello";
            interval.tick().await;
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![generate])
}
