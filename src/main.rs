mod app;
mod handlers;
mod timeshift_lib;
mod ui;

use app::App;
use std::io;
use timeshift_lib::Timeshift;

fn main() -> io::Result<()> {
    let timeshift = Timeshift::new();
    let mut app = App::new(timeshift);
    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
