use serde::{Serialize, Deserialize};
use crate::helpers::url_strip_page;
use url::{Url};
use crate::view::works::work::WorkView;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Ao3Settings {
    pub remember_me: bool,
    pub remember_username: bool,
    pub username: Option<String>,
    pub login_cookie: Option<String>,
    pub base_path: String,
    pub faves: Vec<(String, Url)>,
    pub work_display: WorkView
}

impl Ao3Settings {

    pub fn url_in_faves(&self, mut url: Url) -> bool {
        url_strip_page(&mut url);

        let pos = self.faves.iter().position(|fave| &fave.1 == &url);
        match pos {
            Some(_i) => true,
            None => false
        }
    }

    pub fn toggle_fave(&mut self, title: String, mut url: Url) {
        url_strip_page(&mut url);

        let pos = self.faves.iter().position(|fave| &fave.1 == &url);
        match pos {
            Some(i) => {self.faves.remove(i);},
            None => (self.faves.push((title, url)))
        };
    }
}

impl Default for Ao3Settings {
    fn default() -> Self {
        Ao3Settings {
            remember_me: false,
            remember_username: false,
            username: None,
            login_cookie: None,
            base_path: "".to_string(),
            faves: Vec::new(),
            work_display: WorkView::Short
        }
    }
}