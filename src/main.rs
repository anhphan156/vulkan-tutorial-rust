use app::App;

pub mod app;
pub mod util;

fn main() {
    let mut app = App::new();
    app.main_loop();
}
