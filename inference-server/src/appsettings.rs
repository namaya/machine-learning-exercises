use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AppSettings {
    pub session_cookie_name: String,
    pub session_data_dir: String,
    pub chatbot_path: String,
    pub chatbot_tokenizer_path: String,
}
