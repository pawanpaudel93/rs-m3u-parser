use std::time::Duration;

use m3u_parser::M3uParser;

#[tokio::main]
async fn main() {
    let mut parser = M3uParser::new(Some(Duration::from_secs(5)));
    parser
        .parse_m3u(
            "https://raw.githubusercontent.com/Free-TV/IPTV/master/playlist.m3u8",
            true,
            true,
        )
        .await;
    parser.filter_by("title", vec!["TN Todo Noticias"], "_", false, false);
    parser.sort_by("title", "_", false, false);
    // let json_value = m3u_parser.get_json(true).unwrap();
    let random_stream = parser.get_random_stream(true);
    println!("{:?}", random_stream.unwrap());
    parser.to_file("hello.json", "json")
}
