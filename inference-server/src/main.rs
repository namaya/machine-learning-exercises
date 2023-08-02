#[macro_use]
extern crate rocket;

mod appsettings;
mod chatbot;

use std::fs;
use std::path::Path;

use rocket::fairing::AdHoc;
use rocket::http::{Cookie, CookieJar};
use rocket::response::stream::TextStream;
use rocket::serde::{json::Json, Deserialize};
use rocket::tokio::time::{interval, Duration};
use rocket::{Config, State};
use uuid::Uuid;

use appsettings::AppSettings;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct GenerateBody<'r> {
    prompt: &'r str,
}

#[post("/chat", data = "<body>")]
fn generate(
    body: Json<GenerateBody<'_>>,
    jar: &CookieJar<'_>,
    appsettings: &State<AppSettings>,
) -> TextStream![String] {
    let cookie = jar.get(&appsettings.session_cookie_name);

    let history = if let Some(cookie) = cookie {
        let sess_id_str = cookie.value();
        let path = Path::new(&appsettings.session_data_dir).join(sess_id_str);
        match fs::read_to_string(path) {
            Ok(history) => history,
            Err(error) => {
                panic!("should be 400 error");
            }
        }
    } else {
        let session_id: Uuid = Uuid::new_v4();

        jar.add(Cookie::new(
            appsettings.session_cookie_name.clone(),
            session_id.to_string(),
        ));

        let path = Path::new(&appsettings.session_data_dir).join(session_id.to_string());

        if path.exists() {
            panic!("Writing new session history but session already exists...")
        } else {
            let history = "Some chat session history...";

            fs::write(path, history).expect("error while writing to fs.");

            String::from(history)
        }
    };

    TextStream! {
        let mut interval = interval(Duration::from_secs(1));

        for token in history.split_whitespace() {
            yield token.to_string();
            interval.tick().await;
        }
    }
}

#[launch]
fn rocket() -> _ {
    // let chatbot = chatbot::load().unwrap_or_else(|err| panic!("Failed to load model: {err}"));

    rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![generate])
        .attach(AdHoc::config::<AppSettings>())
}
