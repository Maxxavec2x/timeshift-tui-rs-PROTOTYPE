mod timeshift_lib;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListItem, Widget},
};
use std::io;
use timeshift_lib::Timeshift;

use crate::timeshift_lib::DeviceOrSnapshot;

#[derive(Debug, Default)]
pub struct App {
    //Default permet de set les nombres à 0 et les booléens à false
    exit: bool,
    timeshift_instance: Timeshift,
    current_index: usize,
    current_device_name: String, // Représente le device selectionné
    current_display_screen: String,
    device_names_ordered: Vec<String>, // Ordered list just for the display
}
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.current_index = 0;
        self.current_display_screen = "Device".to_string();
        self.device_names_ordered = self
            .timeshift_instance
            .devices_map_by_name
            .keys()
            .cloned()
            .collect();
        while !self.exit {
            terminal.draw(|frame| self.draw_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw_frame(&self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => self.choose(),
            _ => {}
        }
    }

    fn choose(&mut self) {
        if self.current_display_screen == "Device" {
            // Récupère la clé à l'index actuel
            if let Some(device_name) = self.device_names_ordered.get(self.current_index) {
                self.current_device_name = device_name.clone();
                self.current_display_screen = "Snapshot".to_string();
                self.current_index = 0; // Reset pour les snapshots
            }
        }
    }

    fn select_next(&mut self) {
        let (index, max) = if self.current_display_screen == "Device" {
            (
                self.current_index,
                self.timeshift_instance.devices_map_by_name.keys().len() - 1,
            )
        } else {
            (
                self.current_index,
                self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1,
            )
        };

        if index < max {
            self.current_index += 1;
        }
    }
    fn select_previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }
    fn select_first(&mut self) {
        self.current_index = 0;
    }
    fn select_last(&mut self) {
        let max: usize = if self.current_display_screen == "Device" {
            self.timeshift_instance.devices_map_by_name.keys().len() - 1
        } else {
            self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1
        };
        self.current_index = max;
    }

    fn render_snapshots(&self, area: Rect, buf: &mut Buffer, current_device_name: String) {
        // Conversion en string pour le rendering
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
        let snapshot_list_widget = List::new(items)
            .block(Block::bordered().title("Snapshot List"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        snapshot_list_widget.render(area, buf);
    }

    fn render_devices(&self, area: Rect, buf: &mut Buffer) {
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
            .block(Block::bordered().title("Device List"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        snapshot_list_widget.render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Timeshift TUI ".bold());
        let instructions = Line::from(vec![
            " Delete ".into(),
            " <D> ".blue().bold(),
            " Create ".into(),
            " <C> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        //Paragraph::new().block(block).render(area, buf);
        //self.render_snapshots(area, buf, self.current_device_name.clone());
        block.render(area, buf);
        if self.current_display_screen == "Device" {
            self.render_devices(area, buf);
        } else if self.current_display_screen == "Snapshot" {
            self.render_snapshots(area, buf, self.current_device_name.clone());
        }
        //self.render_snapshots(area, buf, self.current_device_name.clone());
    }
}

fn main() -> io::Result<()> {
    let timeshift = Timeshift::new();
    let mut app: App = App {
        timeshift_instance: timeshift,
        ..Default::default()
    };
    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
