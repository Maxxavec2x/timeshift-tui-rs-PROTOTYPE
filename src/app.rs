use crate::timeshift_lib::Timeshift;
use crate::ui::center;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event;
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
use std::time::Instant;
use throbber_widgets_tui::ThrobberState;
use tui_input::Input;

#[derive(Debug, Default)]
pub struct App {
    pub exit: bool,
    pub timeshift_instance: Timeshift,
    pub current_index: usize,
    pub current_device_name: String,
    pub current_display_screen: Screen,
    pub operation_thread: Option<JoinHandle<Result<(), String>>>, // Threat that I use for creation
    // and deletion
    pub throbber_state: ThrobberState,
    pub current_action: CurrentAction,
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

/// This enum represent the multiple screen of the TUI.
/// THis video opened my mind about how to handle things :
/// https://www.youtube.com/watch?v=z-0-bbc80JM
#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    DeviceScreen,
    SnapshotScreen,
}

/// This enum represent the action that is done by user
#[derive(Debug, Default)]
pub enum CurrentAction {
    #[default]
    Idle,
    SnapshotCreation,
    SnapshotCreationPending,
    SnapshotDeletion,
    SnapshotDeletionConfirmation, // Its not really an action done by the user, but its a state for
                                  // the app
}

impl App {
    pub fn new(timeshift_instance: Timeshift) -> Self {
        Self {
            exit: false,
            timeshift_instance,
            current_index: 0,
            current_device_name: String::new(),
            operation_thread: None,
            throbber_state: ThrobberState::default(),
            ..Default::default()
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.current_index = 0;
        let tick_rate = Duration::from_millis(100); // I set a tickrate so the app can still update
        // even if the user doesn't press a key

        while !self.exit {
            let now = Instant::now();

            self.update();
            terminal.draw(|frame| {
                self.draw_frame(frame);
            })?;

            let timeout = tick_rate
                .checked_sub(now.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)? {
                self.handle_events()?;
            }
        }

        Ok(())
    }

    fn update(&mut self) {
        match self.current_action {
            CurrentAction::SnapshotCreationPending | CurrentAction::SnapshotDeletion => {
                self.throbber_state.calc_next();
                self.check_operation_status();
            }
            _ => (),
        }
    }

    fn draw_frame(&self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());

        let cursor = if let CurrentAction::SnapshotCreation = self.current_action {
            let popup_area = center(
                frame.area(),
                Constraint::Percentage(30),
                Constraint::Length(10),
            );
            frame.render_widget(self, frame.area());
            self.render_creation_popup(popup_area, frame.buffer_mut())
        } else {
            frame.render_widget(self, frame.area());
            None
        };

        if let Some(pos) = cursor {
            frame.set_cursor_position((pos.x, pos.y));
        }
    }

    pub fn update_snapshot_list(&mut self) {
        self.timeshift_instance.update();
    }

    fn check_operation_status(&mut self) {
        if let Some(handle) = self.operation_thread.take() {
            if handle.is_finished() {
                match handle.join() {
                    Ok(Ok(())) => {
                        // Succès
                        self.current_action = CurrentAction::Idle;
                        self.update_snapshot_list();
                        self.current_index = 0;
                    }
                    Ok(Err(e)) => {
                        match self.current_action {
                            CurrentAction::SnapshotDeletion => {
                                // Erreur de suppression
                                panic!("Error deleting snapshot : {:?}", e.to_string());
                            }
                            CurrentAction::SnapshotCreationPending => {
                                panic!("Error creating snapshot : {:?}", e.to_string());
                            }
                            _ => (),
                        }
                    }
                    Err(_) => {
                        // Thread panic
                        panic!("Thread error while operating snapshot");
                    }
                }
            } else {
                // Remettre le handle si pas encore terminé
                self.operation_thread = Some(handle);
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
                match self.current_action {
                    CurrentAction::SnapshotDeletionConfirmation => {
                        self.render_delete_confirmation(
                            area,
                            buf,
                            self.current_device_name.clone(),
                        );
                    }
                    CurrentAction::SnapshotDeletion => {
                        self.render_deletion_progress(area, buf);
                    }
                    CurrentAction::SnapshotCreationPending => {
                        self.render_creation_progress(area, buf);
                    }
                    _ => (),
                }
            }
        }
    }
}
