use crate::app::App;
use crate::ui::{Popup, center};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::Widget,
};
use throbber_widgets_tui::Throbber;

impl App {
    pub fn render_delete_confirmation(
        &self,
        area: Rect,
        buf: &mut Buffer,
        current_device_name: String,
    ) {
        let popup_area = center(area, Constraint::Percentage(30), Constraint::Length(10));
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

    pub fn render_deletion_progress(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = center(area, Constraint::Percentage(30), Constraint::Length(8));

        let popup = Popup::default()
            .title("⏳ Suppression en cours")
            .title_style(Style::default().fg(Color::Cyan).bold())
            .content(Text::from(vec![
                Line::from(""),
                Line::from("Suppression du snapshot...").centered(),
                Line::from(""),
                Line::from("Veuillez patienter").style(Style::default().fg(Color::Gray)),
            ]))
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        popup.render(popup_area, buf);

        let throbber_area = center(popup_area, Constraint::Length(3), Constraint::Length(1));
        let throbber = Throbber::default()
            .label("")
            .style(Style::default().fg(Color::Cyan));
        throbber.render(throbber_area, buf);
    }
}
