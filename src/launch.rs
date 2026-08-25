use open::that;

pub async fn launch_browser(url: &str) -> Result<(), std::io::Error> {
    that(url)
}
