use axum_flash::{IncomingFlashes, Level};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateError {
    level: Level,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]

pub struct TemplateErrors {
    errors: Vec<TemplateError>,
}

impl TemplateErrors {
    fn new(errors: Vec<TemplateError>) -> Self {
        Self { errors }
    }
}

impl From<IncomingFlashes> for TemplateErrors {
    fn from(flash: IncomingFlashes) -> Self {
        Self {
            errors: flash
                .into_iter()
                .map(|(level, message)| TemplateError {
                    level,
                    message: message,
                })
                .collect(),
        }
    }
}
