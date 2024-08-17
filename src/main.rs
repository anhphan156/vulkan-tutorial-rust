use app::App;

pub mod app;

fn main() {
    let mut app = App::new();
    app.main_loop();
}
