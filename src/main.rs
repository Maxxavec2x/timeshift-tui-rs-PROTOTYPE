mod timeshift_lib;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget},
};
use std::io;
use timeshift_lib::Timeshift;

use crate::timeshift_lib::Snapshot;

#[derive(Debug, Default)]
pub struct App {
    //Default permet de set les nombres à 0 et les booléens à false
    exit: bool,
    snapshot_list: Vec<Snapshot>,
}
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc => self.exit = true,
            _ => {}
        }
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

        let main_content = Text::from(
            self.snapshot_list
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );

        Paragraph::new(main_content).block(block).render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut app: App = App {
        snapshot_list: Timeshift::new().snapshots,
        ..Default::default()
    };
    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
