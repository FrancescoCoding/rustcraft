use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn get_asset(name: &str) -> Option<Vec<u8>> {
    Assets::get(name).map(|file| file.data.into_owned())
}
