use crate::app::App;
use crate::app::CurrentAction;
use crate::app::InputMode;
use crate::app::Screen;
use crate::timeshift_lib::Timeshift;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;
use std::thread;
use tui_input::backend::crossterm::EventHandler;

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
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
        // Si on est en mode création, on gère l'input différemment (c'est moche)
        match self.current_action {
            CurrentAction::SnapshotCreation => {
                self.handle_creation_key_event(key_event);
            }
            _ => match key_event.code {
                KeyCode::Char('q') => self.back_or_exit(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(),
                KeyCode::Char('c') => {
                    if let Screen::SnapshotScreen = self.current_display_screen {
                        self.current_action = CurrentAction::SnapshotCreation;
                        self.input_mode = crate::app::InputMode::Editing;
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if let Screen::SnapshotScreen = self.current_display_screen {
                        self.current_action = CurrentAction::SnapshotDeletionConfirmation
                    }
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let CurrentAction::SnapshotDeletionConfirmation = self.current_action {
                        self.delete_current_snapshot();
                    }
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    if let CurrentAction::SnapshotDeletionConfirmation = self.current_action {
                        self.current_action = CurrentAction::Idle;
                    }
                }
                KeyCode::Enter => self.choose(),
                _ => {}
            },
        }
    }

    fn handle_creation_key_event(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Editing => match key_event.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.current_action = CurrentAction::Idle;
                }
                KeyCode::Enter => {
                    // TODO: Improve lifetime thingy.
                    self.current_action = CurrentAction::SnapshotCreationPending;
                    let comment = self.input.value_and_reset();
                    let device_name = self.current_device_name.clone();
                    self.operation_thread = Some(thread::spawn(move || {
                        Timeshift::create_snapshot(comment, &device_name).map_err(|e| e.to_string())
                    }));
                }
                _ => {
                    self.input.handle_event(&Event::Key(key_event));
                }
            },
            InputMode::Normal => match key_event.code {
                // doesnt happend for now
                KeyCode::Enter => {
                    // TODO: Improve lifetime thingy.
                    self.current_action = CurrentAction::SnapshotCreationPending;
                    let comment = self.input.value_and_reset();
                    let device_name = self.current_device_name.clone();
                    self.operation_thread = Some(thread::spawn(move || {
                        Timeshift::create_snapshot(comment, &device_name).map_err(|e| e.to_string())
                    }));
                }
                KeyCode::Esc => {
                    self.current_action = CurrentAction::Idle;
                    self.input.reset();
                }
                _ => {}
            },
        }
    }

    fn back_or_exit(&mut self) {
        match self.current_display_screen {
            Screen::DeviceScreen => self.exit = true,
            Screen::SnapshotScreen => {
                self.input_mode = crate::app::InputMode::Normal;
                self.current_display_screen = Screen::DeviceScreen;
                self.current_index = 0; // Reset pour les snapshots
            }
        }
    }

    fn choose(&mut self) {
        if let Screen::DeviceScreen = self.current_display_screen {
            // Récupère la clé à l'index actuel
            let device_name = self.timeshift_instance.devices_map.keys()[self.current_index]
                .device_name
                .clone();
            self.current_device_name = device_name.clone();
            self.current_display_screen = Screen::SnapshotScreen;
            self.current_index = 0; // Reset pour les snapshots
        }
    }

    pub fn select_next(&mut self) {
        let max = match self.current_display_screen {
            Screen::DeviceScreen => self.timeshift_instance.devices_map_by_name.keys().len() - 1,
            Screen::SnapshotScreen => {
                self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1
            }
        };
        if self.current_index < max {
            self.current_index += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn select_first(&mut self) {
        self.current_index = 0;
    }

    pub fn select_last(&mut self) {
        let max = match self.current_display_screen {
            Screen::DeviceScreen => self.timeshift_instance.devices_map_by_name.keys().len() - 1,
            Screen::SnapshotScreen => {
                self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1
            }
        };
        self.current_index = max;
    }

    fn delete_current_snapshot(&mut self) {
        if let Screen::SnapshotScreen = self.current_display_screen {
            let snapshot_to_delete = &self.timeshift_instance.devices_map_by_name
                [&self.current_device_name.clone()][self.current_index];
            // On créé un thread pour delete le snapshot, et on attend la fin du tread.
            // Pour faire ça, comme la closure capture self, on clone les valeurs utilisé par
            // timeshift (c'est pas le plus opti, mais on est pas à ça près lol), et on les move
            // dans la closure
            let snapshot_name = snapshot_to_delete.clone();
            let current_device = self.current_device_name.clone();
            self.current_action = CurrentAction::SnapshotDeletion;
            self.operation_thread = Some(thread::spawn(move || {
                Timeshift::delete_snapshot(&snapshot_name.name, &current_device)
                    .map_err(|e| e.to_string())
            }));
        }
    }
}
