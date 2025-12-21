mod timeshift_lib;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use derive_setters::Setters;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};
use std::{io, thread};
use timeshift_lib::Timeshift;

#[derive(Debug, Default, Setters)]
struct Popup<'a> {
    //Also stolen from https://ratatui.rs/recipes/render/overwrite-regions/, I still
    //don't understand those lifetime things and i don't want to
    #[setters(into)]
    title: Line<'a>,
    #[setters(into)]
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}

#[derive(Debug, Default)]
pub struct App {
    //Default permet de set les nombres à 0 et les booléens à false
    exit: bool,
    timeshift_instance: Timeshift,
    current_index: usize,
    current_device_name: String, // Représente le device selectionné
    current_display_screen: String,
    show_delete_confirmation: bool,
}
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.show_delete_confirmation = false;
        self.current_index = 0;
        self.current_display_screen = "Device".to_string();
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
            KeyCode::Char('q') => self.back_or_exit(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('d') | KeyCode::Delete => self.show_delete_confirmation = true,
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.delete_current_snapshot();
                self.show_delete_confirmation = false;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.show_delete_confirmation = false;
            }
            KeyCode::Enter => self.choose(),
            _ => {}
        }
    }

    fn delete_current_snapshot(&mut self) {
        if self.current_display_screen == "Snapshot" {
            let snapshot_to_delete = &self.timeshift_instance.devices_map_by_name
                [&self.current_device_name.clone()][self.current_index];
            // On créé un thread pour delete le snapshot, et on attend la fin du tread
            //thread::spawn(|| {
            Timeshift::delete_snapshot(&snapshot_to_delete.name, &self.current_device_name)
                .expect("Erreur deleting snapshot");
            //});

            self.update_snapshot_list();
        }
    }

    fn update_snapshot_list(&mut self) {
        self.timeshift_instance.update();
    }

    fn back_or_exit(&mut self) {
        if self.current_display_screen == "Device" {
            self.exit = true;
        } else {
            self.current_display_screen = "Device".to_string();
            self.current_index = 0; // Reset pour les snapshots
        }
    }

    fn choose(&mut self) {
        if self.current_display_screen == "Device" {
            // Récupère la clé à l'index actuel
            let device_name = self.timeshift_instance.devices_map.keys()[self.current_index]
                .device_name
                .clone();
            self.current_device_name = device_name.clone();
            self.current_display_screen = "Snapshot".to_string();
            self.current_index = 0; // Reset pour les snapshots
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
        let instructions = Line::from(vec![
            " Delete ".into(),
            " <D> ".blue().bold(),
            " Create ".into(),
            " <C> ".blue().bold(),
            " Back ".into(),
            " <Q> ".blue().bold(),
        ]);
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
            .block(
                Block::bordered()
                    .title("Snapshot List")
                    .title_bottom(instructions.centered()),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        snapshot_list_widget.render(area, buf);
    }

    fn render_devices(&self, area: Rect, buf: &mut Buffer) {
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

    fn render_delete_confirmation(
        &self,
        area: Rect,
        buf: &mut Buffer,
        current_device_name: String,
    ) {
        let popup_area = center(
            area,
            Constraint::Percentage(30),
            Constraint::Length(10), // top and bottom border + content
        );
        let snapshot_name =
            &self.timeshift_instance.devices_map_by_name[&current_device_name][self.current_index];

        let popup = Popup::default()
            .title("⚠ Confirmation")
            .title_style(Style::default().fg(Color::Yellow).bold())
            .content(Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    "Voulez-vous vraiment supprimer le snapshot :".into(),
                    snapshot_name.to_string().yellow().bold(),
                    " ?".into(),
                ]),
                Line::from(""),
                Line::from("Cette action est irréversible.").style(Style::default().fg(Color::Red)),
                Line::from(""),
                Line::from(vec![
                    " Confirmer ".into(),
                    " <Y> ".green().bold(),
                    "  Annuler ".into(),
                    " <N/Esc> ".red().bold(),
                ]),
            ]))
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::Black));

        popup.render(popup_area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Timeshift TUI ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);
        block.render(area, buf);

        if self.current_display_screen == "Device" {
            self.render_devices(area, buf);
        } else if self.current_display_screen == "Snapshot" {
            self.render_snapshots(area, buf, self.current_device_name.clone());
            if self.show_delete_confirmation {
                self.render_delete_confirmation(area, buf, self.current_device_name.clone());
            }
        }
    }
}

// Component to make popup that I stole from rattatui documentation : https://ratatui.rs/recipes/render/overwrite-regions/
impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ensure that all cells under the popup are cleared to avoid leaking content
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}

// Another utilitary fonction that i stole from the documentation : https://ratatui.rs/recipes/layout/center-a-widget/
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
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
