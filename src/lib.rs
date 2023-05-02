use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::fs::read_to_string;
use std::time::Duration;
use std::vec;
use url::Url;

#[derive(Debug, Clone)]
struct Tvg {
    id: String,
    name: String,
    url: String,
}

#[derive(Debug, Clone)]
struct Country {
    code: String,
    name: String,
}
#[derive(Debug, Clone)]
struct Language {
    code: String,
    name: String,
}

#[derive(Debug, Clone)]

pub struct Info {
    title: String,
    logo: String,
    url: String,
    category: String,
    tvg: Tvg,
    country: Country,
    language: Language,
    status: String,
}

pub struct M3uParser<'a> {
    pub streams_info: Vec<Info>,
    streams_info_backup: Vec<Info>,
    lines: Vec<String>,
    timeout: Duration,
    enforce_schema: bool,
    check_live: bool,
    useragent: &'a str,
    file_regex: Regex,
    tvg_name_regex: Regex,
    tvg_id_regex: Regex,
    logo_regex: Regex,
    category_regex: Regex,
    title_regex: Regex,
    country_regex: Regex,
    language_regex: Regex,
    tvg_url_regex: Regex,
    streams_regex: Regex,
}

impl<'a> M3uParser<'a> {
    pub fn new(timeout: Option<Duration>) -> M3uParser<'a> {
        let useragent =  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36";
        let timeout = timeout.unwrap_or_else(|| Duration::from_secs(5));
        M3uParser {
            streams_info: vec![],
            streams_info_backup: vec![],
            lines: vec![],
            timeout,
            enforce_schema: true,
            check_live: false,
            useragent,
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
            streams_regex: Regex::new(r"acestream://[a-zA-Z0-9]+").unwrap(),
        }
    }

    fn is_valid_url(&self, url: &str) -> bool {
        match Url::parse(url) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn read_url(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let response = client.get(url).send().await?;
        let content = response.text().await?;
        Ok(content)
    }

    fn get_by_regex(&self, regex: &Regex, content: &str) -> Option<String> {
        match regex.captures(content) {
            Some(captures) => Some(captures[1].trim().to_string()),
            None => None,
        }
    }

    pub async fn parse_m3u(
        &mut self,
        path: &str,
        check_live: bool,     /* = true */
        enforce_schema: bool, /* = true */
    ) {
        let content: String;
        self.check_live = check_live;
        self.enforce_schema = enforce_schema;

        if self.is_valid_url(path) {
            match self.read_url(path).await {
                Ok(url_content) => content = url_content,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            }
        } else {
            match read_to_string(path) {
                Ok(file_content) => content = file_content,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            }
        }
        let lines: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        self.lines = lines;

        if self.lines.len() > 0 {
            self.parse_lines().await;
        } else {
            eprintln!("No content to parse!!!");
        }
    }

    async fn parse_lines(&mut self) {
        let num_lines = self.lines.len();
        self.streams_info.clear();
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap();
        let mut requests = Vec::new();
        for line_num in 0..num_lines {
            if self.lines[line_num].contains("#EXTINF") {
                let request = self.parse_line(line_num, &client);
                requests.push(request);
            }
        }
        let results = futures::future::join_all(requests).await;
        for result in results {
            if let Some(info) = result {
                self.streams_info.push(info.clone());
                self.streams_info_backup.push(info);
            }
        }
        println!("Parsing completed !!!");
    }

    async fn parse_line(&self, line_num: usize, client: &reqwest::Client) -> Option<Info> {
        let line_info = &self.lines[line_num];
        let mut stream_link = String::from("");
        let mut streams_link: Vec<String> = vec![];
        let mut status = String::from("BAD");

        for i in [1, 2].iter() {
            let line = &self.lines[line_num + i];
            let is_acestream = self.streams_regex.is_match(&line);
            if line.len() > 0 && (is_acestream || self.is_valid_url(&line)) {
                streams_link.push(line.to_string());
                if is_acestream {
                    status = String::from("GOOD");
                }
                break;
            } else if line.len() > 0 && self.file_regex.is_match(&line) {
                status = String::from("GOOD");
                streams_link.push(line.to_string());
                break;
            }
        }

        if streams_link.len() > 0 {
            stream_link = streams_link[0].to_string();
        }

        if !line_info.is_empty() && !stream_link.is_empty() {
            let mut info = Info {
                title: "".to_string(),
                logo: "".to_string(),
                url: "".to_string(),
                category: "".to_string(),
                tvg: Tvg {
                    id: "".to_string(),
                    name: "".to_string(),
                    url: "".to_string(),
                },
                country: Country {
                    code: "".to_string(),
                    name: "".to_string(),
                },
                language: Language {
                    code: "".to_string(),
                    name: "".to_string(),
                },
                status,
            };

            // Title
            info.title = self
                .get_by_regex(&self.title_regex, &line_info)
                .unwrap_or_default();

            // Logo
            info.logo = self
                .get_by_regex(&self.logo_regex, &line_info)
                .unwrap_or_default();

            // Url
            info.url = stream_link;

            // Category
            info.category = self
                .get_by_regex(&self.category_regex, &line_info)
                .unwrap_or_default();

            // TVG Information
            let tvg_id = self.get_by_regex(&self.tvg_id_regex, &line_info);
            let tvg_name = self.get_by_regex(&self.tvg_name_regex, &line_info);
            let tvg_url = self.get_by_regex(&self.tvg_url_regex, &line_info);

            info.tvg = Tvg {
                id: tvg_id.unwrap_or_default(),
                name: tvg_name.unwrap_or_default(),
                url: tvg_url.unwrap_or_default(),
            };

            // Country
            if let Some(country) = self.get_by_regex(&self.country_regex, &line_info) {
                let country_obj = celes::Country::from_alpha2(&country);
                let country_name = country_obj.unwrap().long_name;
                info.country = Country {
                    code: country,
                    name: country_name.to_string(),
                };
            }

            // Language
            if let Some(language) = self.get_by_regex(&self.language_regex, &line_info) {
                info.language = Language {
                    code: "".to_string(),
                    name: language,
                };
            }

            if self.check_live && info.status.eq("BAD") {
                match client
                    .get(&info.url)
                    .header("User-Agent", self.useragent)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            info.status = "GOOD".to_string();
                        }
                    }
                    Err(_) => {}
                }
            }
            return Some(info);
        }
        return None;
    }

    pub fn reset_operations(&mut self) {
        self.streams_info = self.streams_info_backup.clone();
    }

    fn get_key_value(&'a self, stream_info: &'a Info, key_0: &str, key_1: &str) -> &str {
        let value = match key_0 {
            "title" => &stream_info.title,
            "logo" => &stream_info.logo,
            "url" => &stream_info.url,
            "category" => &stream_info.category,
            "status" => &stream_info.status,
            "tvg" => match key_1 {
                "id" => &stream_info.tvg.id,
                "name" => &stream_info.tvg.name,
                "url" => &stream_info.tvg.url,
                _ => "",
            },
            "country" => match key_1 {
                "code" => &stream_info.country.code,
                "name" => &stream_info.country.name,
                _ => "",
            },
            "language" => match key_1 {
                "code" => &stream_info.country.code,
                "name" => &stream_info.country.name,
                _ => "",
            },
            _ => "",
        };
        value
    }

    pub fn filter_by(
        &mut self,
        key: &str,
        filters: Vec<&str>,
        key_splitter: &str,
        retrieve: bool,
        nested_key: bool,
    ) {
        let (key_0, key_1) = if nested_key {
            match key.split(key_splitter).collect::<Vec<&str>>()[..] {
                [key0, key1] => (key0, key1),
                _ => {
                    eprintln!("Nested key must be in the format <key><key_splitter><nested_key>");
                    return;
                }
            }
        } else {
            (key, "")
        };

        let valid_keys_0: HashSet<&str> = [
            "title", "logo", "url", "category", "tvg", "country", "language", "status",
        ]
        .iter()
        .copied()
        .collect();

        let valid_keys_1: HashSet<&str> =
            ["", "id", "name", "url", "code"].iter().copied().collect();

        if !valid_keys_0.contains(&key_0) {
            eprintln!("{} key is not present.", key);
            return;
        }

        if !valid_keys_1.contains(&key_1) {
            eprintln!("{} key is not present.", key);
            return;
        }

        if filters.is_empty() {
            eprintln!("Filter word/s missing!!!");
            return;
        }

        let re_filters: Vec<Regex> = filters
            .iter()
            .map(|filter| Regex::new(filter).unwrap())
            .collect();

        self.streams_info = if retrieve {
            let streams_info: Vec<Info> = self
                .streams_info
                .iter()
                .filter(|stream_info| {
                    re_filters.iter().any(|filter| {
                        filter.is_match(self.get_key_value(stream_info, key_0, key_1))
                    })
                })
                .cloned()
                .collect();
            streams_info
        } else {
            let streams_info: Vec<Info> = self
                .streams_info
                .iter()
                .filter(|stream_info| {
                    re_filters.iter().all(|filter| {
                        !filter.is_match(self.get_key_value(stream_info, key_0, key_1))
                    })
                })
                .cloned()
                .collect();
            streams_info
        }
    }

    pub fn sort_by(&mut self, key: &str, key_splitter: &str, asc: bool, nested_key: bool) {
        let (key_0, key_1) = if nested_key {
            match key.split(key_splitter).collect::<Vec<&str>>()[..] {
                [key0, key1] => (key0, key1),
                _ => {
                    eprintln!("Nested key must be in the format <key><key_splitter><nested_key>");
                    return;
                }
            }
        } else {
            (key, "")
        };

        let valid_keys_0: HashSet<&str> = [
            "title", "logo", "url", "category", "tvg", "country", "language", "status",
        ]
        .iter()
        .copied()
        .collect();

        let valid_keys_1: HashSet<&str> =
            ["", "id", "name", "url", "code"].iter().copied().collect();

        if !valid_keys_0.contains(&key_0) {
            eprintln!("{} key is not present.", key);
            return;
        }

        if !valid_keys_1.contains(&key_1) {
            eprintln!("{} key is not present.", key);
            return;
        }

        let mut cloned_streams_info = self.streams_info.clone();

        cloned_streams_info.sort_by(|a, b| {
            let a_value = self.get_key_value(a, key_0, key_1);
            let b_value = self.get_key_value(b, key_0, key_1);

            if asc {
                a_value.cmp(b_value)
            } else {
                b_value.cmp(a_value)
            }
        });

        self.streams_info = cloned_streams_info;
    }
}
