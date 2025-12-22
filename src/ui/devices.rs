use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, Widget},
};

impl App {
    pub fn render_devices(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Choose a device ".into(),
            " <Enter> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold(),
        ]);
        // Conversion en string pour le rendering
        let items: Vec<ListItem> = self
            .timeshift_instance
            .devices_map
            .iter()
            .enumerate()
            .map(|(i, s)| {
                // S est un tuple (Device, Vec<Snapshot>)
                if i == self.current_index {
                    ListItem::from(s.0.to_string()).bg(Color::Blue)
                } else {
                    ListItem::from(s.0.to_string())
                }
            })
            .collect();
        let snapshot_list_widget = List::new(items)
            .block(
                Block::bordered()
                    .title("Device List")
                    .title_bottom(instructions.centered()),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        snapshot_list_widget.render(area, buf);
    }
}
