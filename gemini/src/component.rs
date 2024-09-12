use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::ListItem,
};

use crate::model::ChatMessage;

use crate::model::Sender::{Bot, Split, User};

impl From<&ChatMessage> for ListItem<'_> {
    fn from(value: &ChatMessage) -> Self {
        let lines = match value.sender {
            User => {
                let message = value.message.clone();
                let message_lines = message.split("\n");
                let mut lines = Vec::new();
                let mut line_width = 0;
                for line in message_lines {
                    let line = if line_width == 0 {
                        let line = if value.success {
                            format!("{}{:>width$}", line, "👤", width = 4)
                        } else {
                            format!("{}{:>width$}", line, "❌👤", width = 4)
                        };
                        line_width = line.chars().count();
                        line
                    } else {
                        line.to_owned()
                    };
                    lines.push(
                        Line::from(format!("{:width$}", line, width = line_width))
                            .alignment(Alignment::Right)
                            .style(Style::default().fg(Color::Green)),
                    );
                }
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Right)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
            Bot => {
                let message = value.message.clone();
                let message_lines = message.split("\n");
                let mut lines = Vec::new();
                let mut line_width = 0;
                for line in message_lines {
                    let line = if line_width == 0 {
                        let line = format!("🤖 {}", line);
                        line_width = line.len();
                        line
                    } else {
                        let line = format!("   {}", line);
                        line.to_owned()
                    };
                    lines.push(
                        Line::from(line.to_string())
                            .alignment(Alignment::Left)
                            .style(Style::default().fg(Color::Red)),
                    );
                }
                lines.push(
                    Line::from(value.date_time.format("%H:%M:%S").to_string())
                        .alignment(Alignment::Left)
                        .style(Style::default().fg(Color::Cyan)),
                );
                lines
            }
            Split => {
                vec![Line::from(String::new()).alignment(Alignment::Center)]
            }
        };
        ListItem::new(lines)
    }
}
