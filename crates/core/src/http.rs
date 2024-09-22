use crate::ao3_metadata::Ao3Info;
use crate::context::Context;

use crate::html::{self, scrape_login_csrf};
use crate::settings::Settings;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::cookie::CookieStore;
use reqwest::cookie::Jar;
use reqwest::{Error, Url};
use scraper::Html;
use std::sync::Arc;
use std::fs::File;

const AO3: &str = "https://archiveofourown.org";
const AO3_LOGIN: &str = "https://archiveofourown.org/users/login";
const AO3_FAILED_LOGIN: &str = "The password or user name you entered doesn't match our records.";
const AO3_SUCCESS_LOGIN: &str = "Successfully logged in.";
const AO3_ALREADY_LOGIN: &str = "You are already signed in.";

pub struct HttpClient {
    client: Client,
    pub logged_in: bool,
    cookie_set: bool,
    cookies: Arc<Jar>,
}

pub fn update_session(context: &mut Context) {
    if context.settings.ao3.remember_me {
        let url = AO3.parse::<Url>().unwrap();
        match context.client.cookies.cookies(&url) {
            Some(cookie_str) => {
                context.settings.ao3.login_cookie = Some(cookie_str.to_str().unwrap().to_string())
            }
            None => println!("No cookies available"),
        }
    }
}

pub fn test_login(res: Result<Response, Error>, cookie_set: bool) -> bool {
    let mut logged_in = cookie_set;
    match res {
        Ok(r) => {
            let text = r.text();
            match text {
                Ok(t) => {
                    if t.contains(AO3_FAILED_LOGIN) {
                        logged_in = false;
                    } else if t.contains(AO3_SUCCESS_LOGIN) || t.contains(AO3_ALREADY_LOGIN){
                        logged_in = true;
                    } else {
                        logged_in = false;
                    }
                }
                Err(e) => {
                    println!("There was an error logging in: {}", e);
                    logged_in = false;
                }
            };
        }
        Err(e) => {
            println!("{}", e)
        }
    };
    logged_in
}
impl HttpClient {
    pub fn new(settings: &mut Settings) -> HttpClient {
        let cookie_jar = Jar::default();
        let mut cookie_set = false;

        if settings.ao3.remember_me {
            let url = AO3.parse::<Url>().unwrap();
            match settings.ao3.clone().login_cookie {
                Some(cookie) => {
                    cookie_jar.add_cookie_str(&cookie, &url);
                    cookie_jar.add_cookie_str("user_credentials=1; path=/;", &url);
                    cookie_set = true;
                }
                _ => {}
            }
        }
        let cookies = Arc::new(cookie_jar);
        let client = Client::builder()
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        // Note: having a user cookie set doesn't guarantee we're actually logged in
        // as the cookie may be invalid/expired.
        // TODO - Do this in the background, instead of on startup where it
        let res = client.get(AO3).send();
        let logged_in = test_login(res, cookie_set);
        HttpClient {
            client,
            logged_in,
            cookie_set,
            cookies,
        }
    }

    pub fn get_parse(&self, url: &str) -> Html {
        let res = self.client.get(url).send();

        match res {
            Ok(r) => {
                let text = r.text();
                match text {
                    Ok(t) => return Html::parse_document(&t),
                    Err(_e) => return Html::new_fragment(),
                };
            }
            Err(_e) => return Html::new_fragment(),
        };
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.client.get(url)
    }

    pub fn get_html(&self, url: &str) -> String {
        let res = self.client.get(url).send();
        match res {
            Ok(r) => {
                let text = r.text();
                match text {
                    Ok(t) => return t,
                    Err(e) => {
                        return format!(
                            "There was an error in the response body of {}:\n{}",
                            url, e
                        )
                    }
                };
            }
            Err(e) => {
                println!("{}", e);
                return format!("Error fetching {} - {}", url, e);
            }
        }
    }

    pub fn download_work(&self, work: Ao3Info) {

        let work_url = format!("https://archiveofourown.org/works/{}?view_adult=true", work.id);
        let work_html = self.get_parse(&work_url);
        let download_links = html::scrape_link_list(&work_html, "li.download ul li");
        for link in download_links {
            if link.title == "EPUB" {
                let mut file = File::create(work.download_name()).unwrap();
                let mut res = self.get(&link.location).send().unwrap();
                let _result = res.copy_to(&mut file);
            }
        }
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(url)
    }

    pub fn test_login(&mut self) -> bool {
        let res = self.get(AO3).send();
        if !self.cookie_set {
            return false;
        } else {
            return test_login(res, self.cookie_set);
        }
    }

    pub fn login(&mut self, user: &str, password: &str) {
        let html = self.get_parse(AO3_LOGIN);
        let token = scrape_login_csrf(&html);
        let params = [
            ("user[login]", user),
            ("user[password]", password),
            ("user[remember_me]", "1"),
            ("authenticity_token", &token),
        ];

        let res = self.client.post(AO3_LOGIN).form(&params).send();
        let logged_in = test_login(res, self.cookie_set);
        self.logged_in = logged_in;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Url;

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_httpClientIsCreated_THEN_itWillStoreCookies() {
        // WHEN HttpClient is created
        let mut settings: Settings = Default::default();
        let client = HttpClient::new(&mut settings);
        // THEN it will store cookies
        let url = AO3.parse::<Url>().unwrap();
        // Note: when adding custom cookies, they must include a path
        client.cookies.add_cookie_str("fakeTestCookie=unittest; path=/;", &url);
        assert!(client.cookies.cookies(&url).unwrap().to_str().expect("test cookie").to_string().contains("fakeTestCookie=unittest"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_settingsSetToRememberLogin_WHEN_httpClientIsCreated_THEN_itWillHaveAo3Cookies() {
        // GIVEN Settings set to remember login
        let mut settings: Settings = Default::default();
        settings.ao3.remember_me = true;
        settings.ao3.login_cookie = Some("fakeTestCookie=unittest".to_string());
        // WHEN HttpClient is created
        let client = HttpClient::new(&mut settings);
        // THEN it will have Ao3 cookies
        let url = AO3.parse::<Url>().unwrap();
        assert!(client.cookies.cookies(&url).unwrap().to_str().expect("test cookie").to_string().contains("fakeTestCookie=unittest"));
    }
}
