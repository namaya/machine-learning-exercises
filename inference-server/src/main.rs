#[macro_use]
extern crate rocket;

mod appsettings;
mod chatbot;

use std::fs;
use std::path::Path;
use std::str;

use rocket::fairing::AdHoc;
use rocket::futures::stream::Stream;
use rocket::http::{Cookie, CookieJar};
use rocket::response::stream::TextStream;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use uuid::Uuid;

use appsettings::AppSettings;
use chatbot::Chatbot;

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
fn generate<'a>(
    body: Json<GenerateBody<'a>>,
    jar: &CookieJar<'_>,
    appsettings: &State<AppSettings>,
    chatbot: &'a State<Chatbot>,
) -> TextStream<impl Stream<Item = String> + 'a> {
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
        for await token in chatbot.generate(body.prompt, &history) {
            let token = token.clone();
            yield String::from_utf8(token)
                .unwrap_or_else(|err| {
                    println!("error decoding response -- {}", err);
                    String::from("\n")
                });
        }
    }
}

#[launch]
fn rocket() -> _ {
    let app = rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![generate])
        .attach(AdHoc::config::<AppSettings>());

    let app_settings: AppSettings = app.figment().extract().unwrap();

    let chatbot = Chatbot::load(
        &app_settings.chatbot_path,
        &app_settings.chatbot_tokenizer_path,
    )
    .unwrap_or_else(|err| panic!("Failed to load model: {err}"));

    let app = app.manage(chatbot);

    app
}
