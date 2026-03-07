use ferrous_app::{App, AppMode, Color, FerrousApp};

struct GUIMakerApp;

impl FerrousApp for GUIMakerApp {}

fn main() {
    App::new(GUIMakerApp)
        .with_title("GUIMaker")
        .with_size(1280, 720)
        .with_mode(AppMode::Desktop2D)
        .with_decorations(false)
        .with_background_color(Color::rgb(0.08, 0.08, 0.10))
        .run();
}
