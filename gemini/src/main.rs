mod model;
mod ui;
mod utils;

use anyhow::Result;
use ui::UI;

fn main() -> Result<()> {
    // Setup terminal
    let terminal = ratatui::init();
    let app_result = UI::default().run(terminal);
    ratatui::restore();
    app_result
}
