use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Snr {
    pub paths: Vec<String>,
}

impl Snr {
    pub fn update(&self) {

    }
}
