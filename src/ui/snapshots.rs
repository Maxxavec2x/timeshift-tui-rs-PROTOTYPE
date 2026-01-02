use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Widget},
};
impl App {
    pub fn render_snapshots(&self, area: Rect, buf: &mut Buffer, current_device_name: String) {
        let instructions = Line::from(vec![
            " Delete ".into(),
            " <D> ".blue().bold(),
            " Create ".into(),
            " <C> ".blue().bold(),
            " Back ".into(),
            " <Q> ".blue().bold(),
        ]);

        let items: Vec<ListItem> = self.timeshift_instance.devices_map_by_name
            [&current_device_name]
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i == self.current_index {
                    ListItem::from(s.to_string()).bg(Color::Blue)
                } else {
                    ListItem::from(s.to_string())
                }
            })
            .collect();
        if items.is_empty() {
            let message = Paragraph::new("No snapshots on this device").block(
                Block::bordered()
                    .title("Snapshot List")
                    .title_bottom(instructions.centered()),
            );
            message.render(area, buf);
            return;
        }
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Snapshot List")
                    .title_bottom(instructions.centered()),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        list.render(area, buf);
    }
}
