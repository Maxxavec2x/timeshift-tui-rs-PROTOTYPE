use crate::app::App;
use crate::app::InputMode;
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
        if self.is_creating {
            self.handle_creation_key_event(key_event);
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.back_or_exit(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('c') => {
                if self.current_display_screen == "Snapshot" {
                    self.is_creating = true;
                    self.input_mode = crate::app::InputMode::Editing;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if self.current_display_screen == "Snapshot" {
                    self.show_delete_confirmation = true
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if self.show_delete_confirmation {
                    self.delete_current_snapshot();
                    self.show_delete_confirmation = false;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.show_delete_confirmation = false;
            }
            KeyCode::Enter => self.choose(),
            _ => {}
        }
    }

    fn handle_creation_key_event(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Editing => match key_event.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.is_creating = false;
                }
                KeyCode::Enter => {
                    Timeshift::create_snapshot(
                        self.input.value_and_reset(),
                        &self.current_device_name,
                    );
                    self.is_creating = false;
                    self.update_snapshot_list();
                }
                _ => {
                    self.input.handle_event(&Event::Key(key_event));
                }
            },
            InputMode::Normal => match key_event.code {
                KeyCode::Enter => {
                    Timeshift::create_snapshot(
                        self.input.value_and_reset(),
                        &self.current_device_name,
                    );
                    self.is_creating = false;
                    self.update_snapshot_list();
                }
                KeyCode::Esc => {
                    self.is_creating = false;
                    self.input.reset();
                }
                _ => {}
            },
        }
    }

    fn back_or_exit(&mut self) {
        if self.current_display_screen == "Device" {
            self.exit = true;
        } else {
            self.input_mode = crate::app::InputMode::Normal;
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
    pub fn select_next(&mut self) {
        let max = if self.current_display_screen == "Device" {
            self.timeshift_instance.devices_map_by_name.keys().len() - 1
        } else {
            self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1
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
        let max = if self.current_display_screen == "Device" {
            self.timeshift_instance.devices_map_by_name.keys().len() - 1
        } else {
            self.timeshift_instance.devices_map_by_name[&self.current_device_name].len() - 1
        };
        self.current_index = max;
    }

    fn delete_current_snapshot(&mut self) {
        if self.current_display_screen == "Snapshot" {
            let snapshot_to_delete = &self.timeshift_instance.devices_map_by_name
                [&self.current_device_name.clone()][self.current_index];
            // On créé un thread pour delete le snapshot, et on attend la fin du tread.
            // Pour faire ça, comme la closure capture self, on clone les valeurs utilisé par
            // timeshift (c'est pas le plus opti, mais on est pas à ça près lol), et on les move
            // dans la closure
            let snapshot_name = snapshot_to_delete.clone();
            let current_device = self.current_device_name.clone();
            self.show_delete_confirmation = false;
            self.is_deleting = true;
            self.deletion_thread = Some(thread::spawn(move || {
                Timeshift::delete_snapshot(&snapshot_name.name, &current_device)
                    .map_err(|e| e.to_string())
            }));
        }
    }
}
