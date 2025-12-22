use crate::timeshift_lib::Timeshift;
use crossterm::event::poll;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::widgets::Clear;
use std::io;
use std::thread::JoinHandle;
use std::time::Duration;
use throbber_widgets_tui::ThrobberState;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub timeshift_instance: Timeshift,
    pub current_index: usize,
    pub current_device_name: String,
    pub current_display_screen: String,
    pub show_delete_confirmation: bool,
    pub is_deleting: bool,
    pub deletion_thread: Option<JoinHandle<Result<(), String>>>,
    pub throbber_state: ThrobberState,
}

impl App {
    pub fn new(timeshift_instance: Timeshift) -> Self {
        Self {
            exit: false,
            timeshift_instance,
            current_index: 0,
            current_device_name: String::new(),
            current_display_screen: "Device".to_string(),
            show_delete_confirmation: false,
            is_deleting: false,
            deletion_thread: None,
            throbber_state: ThrobberState::default(),
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.is_deleting = false;
        self.show_delete_confirmation = false;
        self.current_index = 0;
        self.current_display_screen = "Device".to_string();
        while !self.exit {
            if self.is_deleting {
                self.throbber_state.calc_next();
                // un peu ghetto de le mettre là j'imagine, mais au point où on
                // en est...
                self.check_deletion_status();
            }
            terminal.draw(|frame| self.draw_frame(frame))?;
            if poll(Duration::from_millis(20))? {
                self.handle_events()?; // L'appel a event::read est bloquant, on le met donc dans un
                // poll. Le fait que je ne suis pas parti sur une appli multithreadé dès le début
                // conduit à des conneries comme ça
            }
        }
        Ok(())
    }

    fn draw_frame(&self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(self, frame.area());
    }

    pub fn update_snapshot_list(&mut self) {
        self.timeshift_instance.update();
    }

    fn check_deletion_status(&mut self) {
        if let Some(handle) = self.deletion_thread.take() {
            if handle.is_finished() {
                match handle.join() {
                    Ok(Ok(())) => {
                        // Succès
                        self.is_deleting = false;
                        self.update_snapshot_list();
                    }
                    Ok(Err(e)) => {
                        // Erreur de suppression
                        // TODO: Ajouter vrai gestion d'erreur
                        self.is_deleting = false;
                        eprint!("{:?}", e.to_string());
                    }
                    Err(_) => {
                        // Thread panic
                        self.is_deleting = false;
                    }
                }
            } else {
                // Remettre le handle si pas encore terminé
                self.deletion_thread = Some(handle);
            }
        }
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
            if self.is_deleting {
                self.render_deletion_progress(area, buf);
            }
        }
    }
}
