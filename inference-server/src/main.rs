#[macro_use] extern crate rocket;

use std::fs;
use std::path::Path;

use rocket::http::{CookieJar, Cookie};
use rocket::tokio::time::{Duration, interval};
use rocket::response::stream::TextStream;
use rocket::serde::{Deserialize, json::Json};
use uuid::Uuid;

const SESSION_COOKIE_NAME: &str = "chat-session-id";
const SESSION_DATA_DIR: &str = "/tmp/chat-sessions";

#[get("/")]
fn index() -> &'static str { 
    "Hello, world!"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct GenerateBody<'r> {
    prompt: &'r str,
}

#[post("/generate", data="<body>")]
fn generate(body: Json<GenerateBody<'_>>, jar: &CookieJar<'_>) -> TextStream![String] {
    let cookie = jar.get(SESSION_COOKIE_NAME);

    let history = if let Some(cookie) = cookie {
        let sess_id_str = cookie.value();
        let path = Path::new(SESSION_DATA_DIR).join(sess_id_str);
        match fs::read_to_string(path) {
            Ok(history) => history,
            Err(error) => {
                panic!("should be 400 error");
            }
        }
    } else {
        let session_id: Uuid = Uuid::new_v4();

        jar.add(Cookie::new(SESSION_COOKIE_NAME, session_id.to_string()));

        let path = Path::new(SESSION_DATA_DIR).join(session_id.to_string());

        if path.exists() {
            panic!("Writing new session history but session already exists...")
        } else {
            let history = "Some chat session history...";

            fs::write(path, history)
                .expect("error while writing to fs.");

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
    rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![generate])
}
