use reqwest::multipart::Form;

#[derive(Default)]
pub struct Submit {
    pub auth: Option<String>,
    pub base_url: Option<String>,
    pub course_key: Option<String>,
    pub project: Option<String>,
    pub url: Option<String>,

    pub class: Option<String>,
    pub cvs: Option<String>,
    pub login: Option<String>,
    pub otp: Option<String>,

    pub form: Form,
}
