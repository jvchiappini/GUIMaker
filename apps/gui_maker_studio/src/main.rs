use gui_maker_studio::GUIMakerApp;
use ferrous_engine::{App, AppMode, Color as AppColor};

fn main() {
    App::new(GUIMakerApp::default())
        .with_title("GUIMaker")
        .with_size(1280, 720)
        .with_mode(AppMode::Flat2D)
        .with_decorations(false)
        .with_resizable(true)
        .with_vsync(false)
        .with_target_fps(Some(240))
        .with_background_color(AppColor::hex("#1E1E1E"))
        .with_font_bytes(include_bytes!("../assets/fonts/Roboto-Regular.ttf"))
        .run();
}
