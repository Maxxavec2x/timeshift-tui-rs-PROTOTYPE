use crate::timeshift_lib::Timeshift;
use crate::ui::center;
use crossterm::event::poll;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::widgets::Clear;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use std::io;
use std::thread::JoinHandle;
use std::time::Duration;
use throbber_widgets_tui::ThrobberState;
use tui_input::Input;

#[derive(Debug, Default)]
pub struct App {
    pub exit: bool,
    pub timeshift_instance: Timeshift,
    pub current_index: usize,
    pub current_device_name: String,
    pub current_display_screen: Screen,
    pub show_delete_confirmation: bool,
    pub is_deleting: bool,
    pub deletion_thread: Option<JoinHandle<Result<(), String>>>,
    pub throbber_state: ThrobberState,
    pub is_creating: bool,
    pub input_mode: InputMode,
    /// Current value of the input box
    pub input: Input,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

// This enum represent the multiple screen of the TUI.
// THis video opened my mind about how to handle things :
// https://www.youtube.com/watch?v=z-0-bbc80JM
#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    DeviceScreen,
    SnapshotScreen,
}

impl App {
    pub fn new(timeshift_instance: Timeshift) -> Self {
        Self {
            exit: false,
            timeshift_instance,
            current_index: 0,
            current_device_name: String::new(),
            show_delete_confirmation: false,
            is_deleting: false,
            deletion_thread: None,
            throbber_state: ThrobberState::default(),
            ..Default::default()
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.is_deleting = false;
        self.is_creating = false;
        self.show_delete_confirmation = false;
        self.current_index = 0;
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

        // Im doing this here because i need the frame object to manage the cursor, but i don't
        // want to refactor all my logic (yet). I should rewrite the App structure and rendering.
        if self.is_creating {
            let popup_area = center(
                frame.area(),
                Constraint::Percentage(30),
                Constraint::Length(10),
            );

            let cursor = {
                let mut buf = frame.buffer_mut();
                self.render_creation_popup(popup_area, &mut buf)
            };

            if let Some(pos) = cursor {
                frame.set_cursor_position((pos.x, pos.y));
            }
        }
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
                        self.current_index = 0;
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
        match self.current_display_screen {
            Screen::DeviceScreen => self.render_devices(area, buf),
            Screen::SnapshotScreen => {
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
}
