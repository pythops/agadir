use ratatui::Frame;

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    app.render(frame);
}
