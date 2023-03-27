use regex::Regex;
use reqwest::blocking::Client;
use std::error::Error;
use std::fs::read_to_string;
use std::time::Duration;
use url::Url;

struct Tvg {
    id: String,
    name: String,
    url: String,
}

struct Country {
    code: String,
    name: String,
}

struct Language {
    code: String,
    name: String,
}

struct Info {
    title: String,
    logo: String,
    url: String,
    category: String,
    tvg: Tvg,
    country: Country,
    language: Language,
}
pub struct M3uParser {
    streams_info: Vec<Info>,
    streams_info_backup: Vec<Info>,
    lines: Vec<String>,
    timeout: Duration,
    enforce_schema: bool,
    check_live: bool,
    useragent: String,
    content: String,
    file_regex: Regex,
    tvg_name_regex: Regex,
    tvg_id_regex: Regex,
    logo_regex: Regex,
    category_regex: Regex,
    title_regex: Regex,
    country_regex: Regex,
    language_regex: Regex,
    tvg_url_regex: Regex,
}

impl M3uParser {
    pub fn new(timeout: Option<Duration>) -> M3uParser {
        let useragent =  String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36");
        let timeout = timeout.unwrap_or_else(|| Duration::from_secs(5));
        M3uParser {
            streams_info: vec![],
            streams_info_backup: vec![],
            lines: vec![],
            timeout,
            enforce_schema: true,
            check_live: false,
            useragent,
            content: String::from(""),
            file_regex: Regex::new(
                r#"^[a-zA-Z]:\\((?:.*?\\)*).*\.[\d\w]{3,5}$|^(/[^/]*)+/?.[\d\w]{3,5}$"#,
            )
            .unwrap(),
            tvg_name_regex: Regex::new(r#"tvg-name="(.*?)""#).unwrap(),
            tvg_id_regex: Regex::new(r#"tvg-id="(.*?)""#).unwrap(),
            logo_regex: Regex::new(r#"tvg-logo="(.*?)""#).unwrap(),
            category_regex: Regex::new(r#"group-title="(.*?)""#).unwrap(),
            title_regex: Regex::new(r#",([^",]+)$"#).unwrap(),
            country_regex: Regex::new(r#"tvg-country="(.*?)""#).unwrap(),
            language_regex: Regex::new(r#"tvg-language="(.*?)""#).unwrap(),
            tvg_url_regex: Regex::new(r#"tvg-url="(.*?)""#).unwrap(),
        }
    }

    fn is_valid_url(&self, url: &str) -> bool {
        match Url::parse(url) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn read_url(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let response = client.get(url).send()?;
        let content = response.text()?;
        Ok(content)
    }

    pub fn parse_m3u(
        &mut self,
        path: &str,
        check_live: bool,     /* = true */
        enforce_schema: bool, /* = true */
    ) {
        self.check_live = if check_live { check_live } else { true };
        self.enforce_schema = if enforce_schema { enforce_schema } else { true };

        if self.is_valid_url(path) {
            match self.read_url(path) {
                Ok(content) => self.content = content,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            }
        } else {
            match read_to_string(path) {
                Ok(content) => self.content = content,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            }
        }
        let lines: Vec<String> = self
            .content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        self.lines = lines;

        if self.lines.len() > 0 {
            self.parse_lines();
        } else {
            eprintln!("No content to parse!!!");
        }
    }

    fn parse_lines(&self) {}
}
