use std::path::PathBuf;

use async_stream::stream;
use llm::{load_progress_callback_stdout, models::Bloom, InferenceError, LoadError};
use llm::{InferenceFeedback, KnownModel};
use rocket::futures::stream::Stream;

pub struct Chatbot {
    pub model: Bloom,
}

impl<'a> Chatbot {
    pub fn load(path: &str, tokenizer_path: &str) -> Result<Chatbot, LoadError> {
        let model_path = PathBuf::from(path);
        let tokenizer = PathBuf::from(tokenizer_path);

        let model = llm::load::<Bloom>(
            &model_path,
            llm::TokenizerSource::HuggingFaceTokenizerFile(tokenizer.clone()),
            Default::default(),
            load_progress_callback_stdout,
        )?;

        Ok(Chatbot { model })
    }

    pub fn generate(
        &'a self,
        prompt: &str,
        session_history: &str,
    ) -> impl Stream<Item = Vec<u8>> + 'a {
        let mut session = self.model.start_session(Default::default());

        // prefill prompt
        let res = session.feed_prompt(&self.model, prompt, &mut Default::default(), |token| {
            Ok::<InferenceFeedback, InferenceError>(llm::InferenceFeedback::Continue)
        });

        stream! {
            if let Err(error) = res {
                println!("Encounted error {}", error);
                return;
            }

            let mut token = match session.infer_next_token(
                &self.model,
                &Default::default(),
                &mut Default::default(),
                &mut rand::thread_rng(),
            ) {
                Ok(token) => token,
                Err(error) => match error {
                    InferenceError::ContextFull => vec![],
                    InferenceError::EndOfText => vec![],
                    InferenceError::TokenizationFailed(tk_error) => {
                        println!("{}", tk_error);
                        vec![]
                    },
                    InferenceError::UserCallback(_) => vec![],
                }
            };

            while !token.is_empty() {
                yield token;

                token = match session.infer_next_token(
                    &self.model,
                    &Default::default(),
                    &mut Default::default(),
                    &mut rand::thread_rng(),
                ) {
                    Ok(token) => token,
                    Err(error) => match error {
                        InferenceError::ContextFull => vec![],
                        InferenceError::EndOfText => vec![],
                        InferenceError::TokenizationFailed(tk_error) => {
                            println!("{}", tk_error);
                            vec![]
                        },
                        InferenceError::UserCallback(_) => vec![],
                    }
                }
            }
        }
    }
}
