use std::time::Duration;

use m3u_parser::M3uParser;

#[tokio::main]
async fn main() {
    let mut m3u_parser = M3uParser::new(Some(Duration::from_secs(5)));
    m3u_parser
        .parse_m3u(
            "https://raw.githubusercontent.com/Free-TV/IPTV/master/playlist.m3u8",
            true,
            true,
        )
        .await
}
