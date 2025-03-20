//! # M3u Parser
//!
//! A library for parsing and manipulating M3U files.

mod language;

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::time::Duration;
use std::vec;
use url::Url;

/// Struct representing the Tvg information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tvg {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// Struct representing the Country information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub code: String,
    pub name: String,
}

/// Struct representing the Language information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
}

/// Struct representing the stream information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub logo: String,
    pub url: String,
    pub category: String,
    pub tvg: Tvg,
    pub country: Country,
    pub language: Language,
    pub status: String,
}

/// M3U Parser struct for parsing and manipulating M3U files.
pub struct M3uParser<'a> {
    pub streams_info: Vec<Info>,
    streams_info_backup: Vec<Info>,
    lines: Vec<String>,
    timeout: Duration,
    enforce_schema: bool,
    check_live: bool,
    useragent: &'a str,
    file_regex: Lazy<Regex>,
    tvg_name_regex: Lazy<Regex>,
    tvg_id_regex: Lazy<Regex>,
    logo_regex: Lazy<Regex>,
    category_regex: Lazy<Regex>,
    title_regex: Lazy<Regex>,
    country_regex: Lazy<Regex>,
    language_regex: Lazy<Regex>,
    tvg_url_regex: Lazy<Regex>,
    streams_regex: Lazy<Regex>,
}

impl<'a> M3uParser<'a> {
    /// Creates a new instance of M3uParser.
    ///
    /// # Arguments
    ///
    /// * `timeout` - An optional `Duration` specifying the timeout for network requests.
    ///               If not provided, a default timeout of 5 seconds is used.
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
            file_regex: Lazy::new(|| {
                Regex::new(r#"^[a-zA-Z]:\\((?:.*?\\)*).*\.[\d\w]{3,5}$|^(/[^/]*)+/?.[\d\w]{3,5}$"#)
                    .unwrap()
            }),
            tvg_name_regex: Lazy::new(|| Regex::new(r#"tvg-name="(.*?)""#).unwrap()),
            tvg_id_regex: Lazy::new(|| Regex::new(r#"tvg-id="(.*?)""#).unwrap()),
            logo_regex: Lazy::new(|| Regex::new(r#"tvg-logo="(.*?)""#).unwrap()),
            category_regex: Lazy::new(|| Regex::new(r#"group-title="(.*?)""#).unwrap()),
            title_regex: Lazy::new(|| Regex::new(r#",([^",]+)$"#).unwrap()),
            country_regex: Lazy::new(|| Regex::new(r#"tvg-country="(.*?)""#).unwrap()),
            language_regex: Lazy::new(|| Regex::new(r#"tvg-language="(.*?)""#).unwrap()),
            tvg_url_regex: Lazy::new(|| Regex::new(r#"tvg-url="(.*?)""#).unwrap()),
            streams_regex: Lazy::new(|| Regex::new(r"acestream://[a-zA-Z0-9]+").unwrap()),
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

    fn save_file(&self, filename: &str, data: &[u8]) {
        let mut file = File::create(filename).unwrap();
        file.write(data).unwrap();
        println!("Saved to file: {}", filename);
    }

    fn get_by_regex(&self, regex: &Regex, content: &str) -> Option<String> {
        match regex.captures(content) {
            Some(captures) => Some(captures[1].trim().to_string()),
            None => None,
        }
    }

    /// Parses the specified M3U playlist file or URL.
    ///
    /// # Arguments
    ///
    /// * `path` - The path or URL of the M3U playlist.
    /// * `check_live` - A boolean indicating whether to check the availability of streams.
    ///                  If set to `true`, the parser will make a request to each stream URL to check its status.
    /// * `enforce_schema` - A boolean indicating whether to enforce the M3U schema.
    ///                      If set to `true`, only valid M3U entries will be parsed.
    pub async fn parse_m3u(&mut self, path: &str, check_live: bool, enforce_schema: bool) {
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

        if !self.lines.is_empty() {
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
        let mut stream_link = String::new();
        let mut streams_link: Vec<String> = vec![];
        let mut status = String::from("BAD");

        for i in [1, 2].iter() {
            let line = &self.lines[line_num + i];
            let is_acestream = self.streams_regex.is_match(&line);
            if !line.is_empty() && (is_acestream || self.is_valid_url(&line)) {
                streams_link.push(line.to_string());
                if is_acestream {
                    status = String::from("GOOD");
                }
                break;
            } else if !line.is_empty() && self.file_regex.is_match(&line) {
                status = String::from("GOOD");
                streams_link.push(line.to_string());
                break;
            }
        }

        if !streams_link.is_empty() {
            stream_link = streams_link[0].to_string();
        }

        if !line_info.is_empty() && !stream_link.is_empty() {
            let mut info = Info {
                title: String::new(),
                logo: String::new(),
                url: String::new(),
                category: String::new(),
                tvg: Tvg {
                    id: String::new(),
                    name: String::new(),
                    url: String::new(),
                },
                country: Country {
                    code: String::new(),
                    name: String::new(),
                },
                language: Language {
                    code: String::new(),
                    name: String::new(),
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
                let mut country_name = "";
                if let Ok(country_obj) = celes::Country::from_alpha2(&country) {
                    country_name = country_obj.long_name;
                }
                info.country = Country {
                    code: country,
                    name: country_name.to_string(),
                };
            }

            // Language
            if let Some(language) = self.get_by_regex(&self.language_regex, &line_info) {
                let language_lower = language.to_lowercase();
                let country_code = language::get_language_code(&language_lower);
                info.language = Language {
                    code: country_code.to_owned().to_string(),
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

    fn get_m3u_content(&self) -> String {
        if self.streams_info.is_empty() {
            return String::new();
        }

        let content: Vec<String> = self
            .streams_info
            .iter()
            .map(|stream_info| {
                let mut line = String::from("#EXTINF:-1");

                macro_rules! append_attribute {
                    ($attr:expr, $value:expr) => {
                        if !$value.is_empty() {
                            line.push_str(&format!(" {}=\"{}\"", $attr, $value));
                        }
                    };
                }

                append_attribute!("tvg-id", stream_info.tvg.id);
                append_attribute!("tvg-name", stream_info.tvg.name);
                append_attribute!("tvg-url", stream_info.tvg.url);
                append_attribute!("tvg-logo", stream_info.logo);
                append_attribute!("tvg-country", stream_info.country.code);
                append_attribute!("tvg-language", stream_info.language.name);
                append_attribute!("group-title", stream_info.category);

                if !stream_info.title.is_empty() {
                    line.push_str(&format!(",{}", stream_info.title));
                }

                format!("{}\n{}", line, stream_info.url)
            })
            .collect();
        ["#EXTM3U".to_string(), content.join("\n")].join("\n")
    }

    /// Resets the operations of the M3uParser by restoring the backup of stream information.
    ///
    /// This function restores the original state of the M3uParser by replacing the current
    /// stream information with the backup. This can be useful when you want to undo any
    /// modifications or filtering operations applied to the stream information.
    ///
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

    /// Filters the stream information based on the specified key and filters.
    ///
    /// This function applies filtering operations to the stream information based on the provided key
    /// and filters. The key represents the attribute of the stream information that will be filtered,
    /// and the filters specify the conditions that the attribute should match. The function allows
    /// filtering based on nested keys and provides options to retrieve or exclude the matching
    /// stream information.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key to filter by. Valid values are: "title", "logo", "url", "category",
    ///   "tvg", "country", "language", and "status".
    /// * `filters` - A vector of filter strings. The stream information will be filtered based on
    ///   these conditions.
    /// * `key_splitter` - The delimiter used to split the key for nested filtering. Set it to an empty
    ///   string (`""`) if nested filtering is not required.
    /// * `retrieve` - A boolean value indicating whether to retrieve the matching stream information
    ///   (`true`) or exclude it from the result (`false`).
    /// * `nested_key` - A boolean value indicating whether the key represents a nested key. If `true`,
    ///   the key will be split using the `key_splitter`, and filtering will be applied to the nested
    ///   key. If `false`, the key will be treated as a single key for filtering.
    ///
    /// # Panics
    ///
    /// The function will panic in the following scenarios:
    ///
    /// * If the nested key is provided but not in the format `<key><key_splitter><nested_key>`.
    /// * If the provided key is not one of the valid keys ("title", "logo", "url", "category",
    ///   "tvg", "country", "language", "status").
    ///
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

    /// Sorts the stream information based on the specified key and sorting options.
    ///
    /// This function sorts the stream information based on the provided key and sorting options. The key
    /// represents the attribute of the stream information that will be used for sorting. The function
    /// allows sorting based on nested keys and provides options to specify the sorting order.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key to sort by. Valid values are: "title", "logo", "url", "category",
    ///   "tvg", "country", "language", and "status".
    /// * `key_splitter` - The delimiter used to split the key for nested sorting. Set it to an empty
    ///   string (`""`) if nested sorting is not required.
    /// * `asc` - A boolean value indicating the sorting order. If `true`, the stream information will be
    ///   sorted in ascending order based on the specified key. If `false`, the stream information will
    ///   be sorted in descending order.
    /// * `nested_key` - A boolean value indicating whether the key represents a nested key. If `true`,
    ///   the key will be split using the `key_splitter`, and sorting will be applied to the nested key.
    ///   If `false`, the key will be treated as a single key for sorting.
    ///
    /// # Panics
    ///
    /// The function will panic in the following scenarios:
    ///
    /// * If the nested key is provided but not in the format `<key><key_splitter><nested_key>`.
    /// * If the provided key is not one of the valid keys ("title", "logo", "url", "category",
    ///   "tvg", "country", "language", "status").
    ///
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

    /// Removes stream information based on the specified file extensions.
    ///
    /// This function removes stream information based on the file extensions specified in the `extensions`
    /// parameter. It internally calls the `filter_by` function with the "url" attribute as the key and
    /// filters the stream information that matches any of the provided extensions.
    ///
    /// # Arguments
    ///
    /// * `extensions` - A vector of file extensions to be removed. Each extension should be a string.
    ///
    pub fn remove_by_extension(&mut self, extensions: Vec<&str>) {
        self.filter_by("url", extensions, "-", false, false)
    }

    /// Retrieves stream information based on the specified file extensions.
    ///
    /// This function retrieves stream information based on the file extensions specified in the `extensions`
    /// parameter. It internally calls the `filter_by` function with the "url" attribute as the key and
    /// filters the stream information that matches any of the provided extensions.
    ///
    /// # Arguments
    ///
    /// * `extensions` - A vector of file extensions to be retrieved. Each extension should be a string.
    ///
    pub fn retrieve_by_extension(&mut self, extensions: Vec<&str>) {
        self.filter_by("url", extensions, "-", true, false)
    }

    /// Removes stream information based on the specified categories.
    ///
    /// This function removes stream information based on the categories specified in the `extensions`
    /// parameter. It internally calls the `filter_by` function with the "category" attribute as the key
    /// and filters out the stream information that matches any of the provided categories.
    ///
    /// # Arguments
    ///
    /// * `categories` - A vector of categories to be removed. Each category should be a string.
    ///
    pub fn remove_by_category(&mut self, extensions: Vec<&str>) {
        self.filter_by("category", extensions, "-", false, false)
    }

    /// Retrieves stream information based on the specified categories.
    ///
    /// This function retrieves stream information based on the categories specified in the `extensions`
    /// parameter. It internally calls the `filter_by` function with the "category" attribute as the key
    /// and filters the stream information that matches any of the provided categories.
    ///
    /// # Arguments
    ///
    /// * `categories` - A vector of categories to be retrieved. Each category should be a string.
    ///
    pub fn retrieve_by_category(&mut self, extensions: Vec<&str>) {
        self.filter_by("category", extensions, "-", true, false)
    }

    /// Retrieves the stream information in JSON format.
    ///
    /// This function returns the stream information in JSON format. The JSON can be either
    /// pretty-formatted or compact depending on the `preety` parameter.
    ///
    /// # Arguments
    ///
    /// * `pretty` - A boolean indicating whether to format the JSON output in a pretty, human-readable way.
    ///
    /// # Returns
    ///
    /// A `serde_json::Result<String>` representing the JSON output. If the serialization to JSON is successful,
    /// the result will contain the JSON string. Otherwise, an error indicating the reason for the failure
    /// will be returned.
    ///
    pub fn get_json(&self, preety: bool) -> serde_json::Result<String> {
        let streams_json: String;
        if preety {
            streams_json = serde_json::to_string_pretty(&self.streams_info)?;
        } else {
            streams_json = serde_json::to_string(&self.streams_info)?;
        }
        Ok(streams_json)
    }

    /// Retrieves a vector containing all stream information.
    ///
    /// This function returns a deep clone of the internal `streams_info` vector, which
    /// contains all the stream information.
    ///
    /// # Returns
    ///
    /// A `Vec<Info>` containing all stream information. If there is no stream information
    /// available, an empty vector will be returned.
    ///
    pub fn get_vector(&self) -> Vec<Info> {
        self.streams_info.clone()
    }

    /// Retrieves a random stream from the available stream information.
    ///
    /// This function randomly selects a stream from the available stream information.
    /// The `random_shuffle` parameter determines whether to shuffle the stream information
    /// before selecting a random stream. If the stream information is empty, `None` will be returned.
    ///
    /// # Arguments
    ///
    /// * `random_shuffle` - A boolean indicating whether to shuffle the stream information before
    ///                      selecting a random stream.
    ///
    /// # Returns
    ///
    /// An `Option<&Info>` representing the randomly selected stream. If a stream is successfully
    /// selected, the result will contain a reference to the stream. Otherwise, if the stream
    /// information is empty, `None` will be returned.
    ///
    pub fn get_random_stream(&mut self, random_shuffle: bool) -> Option<&Info> {
        if self.streams_info.is_empty() {
            eprintln!("No streams information so could not get any random stream.");
            return None;
        }
        let mut rng = thread_rng();
        let stream_infos = &mut self.streams_info[..];
        if random_shuffle {
            stream_infos.shuffle(&mut rng);
        }
        Some(stream_infos.choose(&mut rng).unwrap())
    }

    /// Saves the stream information to a file in the specified format.
    ///
    /// This function saves the stream information to a file with the given `filename` and `format`.
    /// If the `filename` already contains a file extension, it will be used as the format. Otherwise,
    /// the `format` parameter will be used as the file extension.
    ///
    /// The supported formats are "json" and "m3u". For "json" format, the stream information will be
    /// saved as a JSON string in a pretty printed format. For "m3u" format, the stream information will
    /// be saved as an M3U playlist.
    ///
    /// # Arguments
    ///
    /// * `filename` - A string representing the name of the file to be saved. If the file already exists,
    ///                it will be overwritten.
    /// * `format` - A string representing the format in which the stream information should be saved. If
    ///              the `filename` already contains a file extension, it will be used as the format.
    ///              Otherwise, the `format` parameter will be used as the file extension.
    ///
    /// # Panics
    ///
    /// This function panics if there is an error while converting the stream information to the specified format
    /// or if there is an error while saving the file.
    pub fn to_file(&self, filename: &str, format: &str) {
        let format = if filename.contains(".") {
            filename.split(".").last().unwrap_or(format)
        } else {
            format
        };

        let filename = match filename.to_lowercase().ends_with(format) {
            true => filename.to_owned(),
            false => format!("{}.{}", filename, format),
        };

        if self.streams_info.is_empty() {
            eprintln!("Either parsing is not done or no stream info was found after parsing !!!");
            return;
        }

        println!("Saving to file: {}", filename);
        match format {
            "json" => {
                let content = self.get_json(true).unwrap();
                self.save_file(filename.as_str(), content.as_bytes());
            }
            "m3u" => {
                let content = self.get_m3u_content();
                self.save_file(filename.as_str(), content.as_bytes());
            }
            _ => eprintln!("Unrecognised format!!!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::M3uParser;

    #[tokio::test]
    async fn test_m3u_parser() {
        let mut parser = M3uParser::new(Some(Duration::from_secs(5)));
        parser
            .parse_m3u(
                "https://iptv-org.github.io/iptv/index.country.m3u",
                true,
                true,
            )
            .await;

        parser.filter_by("title", vec!["Metro TV"], "_", false, false);
        parser.sort_by("title", "_", false, false);

        assert!(
            !parser
                .streams_info
                .iter()
                .any(|info| info.title == "Metro TV"),
            "Metro TV is available as a title"
        );

        let random_stream = parser.get_random_stream(true);
        assert!(random_stream.is_some(), "Random stream should be available");

        let file_path = "hello.m3u";
        parser.to_file(file_path, "m3u");

        // Assert that the file exists
        assert!(fs::metadata(file_path).is_ok(), "Output file should exist");

        // Clean up the temporary file
        if let Err(err) = fs::remove_file(file_path) {
            eprintln!("Failed to remove file: {}", err);
        }
    }
}
