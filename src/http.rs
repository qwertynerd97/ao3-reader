use scraper::Selector;
use scraper::Html;
use reqwest::{Url, Error};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::cookie::Jar;
use reqwest::cookie::CookieStore;
use std::sync::Arc;
use std::convert::Into;
use crate::settings::Settings;
use crate::app::Context;
use crate::helpers::decode_entities;
use serde::{Serialize, Deserialize};

const AO3: &str = "https://archiveofourown.org";

pub struct HttpClient{
    client: Client,
    logged_in: bool,
    cookie_set: bool,
    cookies: Arc<Jar>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link{
    pub title: String,
    pub location: String
}

pub fn list_to_str(list: &Vec<Link>, sep: &str) -> String {
    let mut temp = Vec::new();
    for link in list {
        temp.push(link.title.clone());
    }
    temp.join(sep)
}

pub fn update_session(context: &mut Context) {
    if context.settings.ao3.remember_me {
        let url = AO3.parse::<Url>().unwrap();
        match context.client.cookies.cookies(&url) {
            Some(cookie_str) => context.settings.ao3.login_cookie = Some(cookie_str.to_str().unwrap().to_string()),
            None => println!("No cookies available")
        }
    }
}

pub fn test_login(res: Result<Response, Error>, cookie_set: bool) -> bool {
    let mut logged_in = cookie_set;
    match res {
        Ok(r) => {
            let cookies = r.cookies();
            for cookie in cookies {
                if cookie.name() == "user_credentials" && cookie.value() == "1" {
                    logged_in = true;
                }
                if cookie.name() == "user_credentials" && cookie.value() == "" {
                    logged_in = false;
                }
            }
        },
        Err(e) => { println!("{}", e) }
    };
    logged_in
}

pub fn scrape_csrf(frag: &Html) -> String {
    let token = Selector::parse(r#"input[name="authenticity_token"]"#).unwrap();
    let input = frag.select(&token).next().unwrap();
    input.value().attr("value").unwrap().to_string()
}

pub fn scrape(frag: &Html, select: &str) -> String {
    let selector = Selector::parse(select).unwrap();
    match frag.select(&selector).next() {
        Some(el) => {
            let raw = el.inner_html();
            let trimmed = raw.trim();
            let clean = decode_entities(trimmed).into_owned();
            return clean
        },
        None => {
            println!("error trying to scrape {}", select);
            return "#####".to_string()
        }
    };
}

pub fn scrape_link_list(frag: &Html, select: &str) -> Vec<Link> {
    let selector = Selector::parse(select).unwrap();
    let elems = frag.select(&selector);

    let mut results = Vec::new();
    for el in elems {
        let raw_title = el.inner_html();
        let trimmed = raw_title.trim();
        let title = decode_entities(trimmed).into_owned();
        let location = el.value().attr("href").unwrap_or("").to_string();
        results.push(Link{title, location});
    }

    results

}

pub fn scrape_link(frag: &Html, select: &str) -> Link {
    let selector = Selector::parse(select).unwrap();
    match frag.select(&selector).next() {
        Some(el) => {
            let raw_title = el.inner_html();
            let trimmed = raw_title.trim();
            let title = decode_entities(trimmed).into_owned();
            let location = el.value().attr("href").unwrap_or("").to_string();
            return Link{title, location}; 
        }
        None => {
            return Link{title: "####".to_string(), location: "".to_string()};
        }
    };
}

pub fn scrape_many(frag: &Html, select: &str) -> Vec<String> {
    let selector = Selector::parse(select).unwrap();
    let elems = frag.select(&selector);

    let mut results = Vec::new();
    for el in elems {
        results.push(el.inner_html());
    }

    results
}

pub fn scrape_many_outer(frag: &Html, select: &str) -> Vec<String> {
    let selector = Selector::parse(select).unwrap();
    let elems = frag.select(&selector);

    let mut results = Vec::new();
    for el in elems {
        results.push(el.html());
    }

    results
}


pub fn scrape_outer(frag: &Html, select: &str) -> String {
    let selector = Selector::parse(select).unwrap();
    match frag.select(&selector).next() {
        Some(el) => return el.inner_html(),
        None => return "#####".to_string()
    };
}

pub fn scrape_inner(frag: String, select: &str) -> String {
    let html = Html::parse_fragment(&frag);
    let selector = Selector::parse(select).unwrap();
    html.select(&selector).next().unwrap().html()
}

impl HttpClient {
    pub fn new(settings: &mut Settings) -> HttpClient {
        let cookie_jar = Jar::default();
        let mut cookie_set = false;

        let url = AO3.parse::<Url>().unwrap();
        // cookie_jar.add_cookie_str("_otwarchive_session=NW9hYmtlem9LcjE0dlE4WWJXUUpjTGxZRUhCV0sxSmNMaXJxRi81Mjd5d2owTU13ODVRM2g5REtFcjgvS1VNb2VtYktFN3djWWJBYkwyZ3JzUjhoMkNNVU5vM080bDRzVFN6K2xVWXhVRlUvQzZYT0RSQVkxWnpxWlBmb0ZlbVNMN3Q3VEFFZUdsRjFxQzNFV1N4RmRrb0lpRVUxdS8vZG04WnQxOFFKVFBBbkI5cy9UVGJiTWRhalQ1Vkh4cW50YjdiaHRFL0JPQUVLMDEyRkpBRGxQYVcxRlI2N0FieXh6VHU4a0k3TEpKc1h0VnA3RGRLb0VVcmYvN3A3VUh5K0NYSTZQOEtvRjhvbDRibExmanltNlp0ZmxzRG55emtYeENRc3R4ZE0vL2tkd0VDS1VGTHYybERyaktEVmxteFQ2YkdUa1NRdGVHNWZNcXZDc0I3OTIyNzk1WmV1WGU1ajQ3bWNPdGJrLzFPa3VTVk81RjYvNzd6aW5vd0Y2MlRFaDYwME4xbnFQWVFnSzRNRTQ5ZGZrdz09LS02VVpFYmZRcW9tOVgzbjFjckRtbkh3PT0%3D--fc6de192acb90c3cf4652bafe8dcf2848fae70f9; path=/; expires=Fri, 02 Jul 2021 00:46:22 GMT; HttpOnly", &url);
        // cookie_jar.add_cookie_str("user_credentials=1; path=/;", &url);
        // cookie_set = true;

        // let get_cookies = cookie_jar.cookies(&url);
        // println!("{:?}", get_cookies);

        if settings.ao3.remember_me {
            let url = AO3.parse::<Url>().unwrap();
            match settings.ao3.clone().login_cookie {
                Some(cookie) => {
                    cookie_jar.add_cookie_str(&cookie, &url);
                    cookie_jar.add_cookie_str("user_credentials=1; path=/;", &url);
                    cookie_set = true;
                },
                _ => {}
            }
        }
        let cookies = Arc::new(cookie_jar);
        let client = Client::builder()
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();
    
        // Note: having a user cookie set doesn't guarantee we're actually logged in
        // as the cookie may be invalid/expired. We'll test in a later step
        HttpClient {
            client,
            logged_in: false,
            cookie_set,
            cookies
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
            },
            Err(_e) => {
                return Html::new_fragment()
                }
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
                    Err(e) => return format!("There was an error in the response body of {}:\n{}", url, e),
                };
            },
            Err(e) => {
                println!("{}", e);
                return format!("Error fetching {} - {}", url, e);
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
            println!("testing login");
            return test_login(res, self.cookie_set);
        }
        
    }

    pub fn login(&mut self, user: &str, password: &str) {
        let html = self.get_parse("https://archiveofourown.org");
        let token = scrape_csrf(&html);
        let params=[("user[login]", user), 
                    ("user[password]", password), 
                    ("commit", "Log+In"), 
                    ("utf8", "✓"),
                    ("authenticity_token", &token) ];

        let res = self.client.post("https://archiveofourown.org/users/login")
            .form(&params)
            .send();
        let logged_in = test_login(res, self.cookie_set);
        println!("trying to log in");
        self.logged_in = logged_in;

    }
}