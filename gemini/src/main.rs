mod model;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType},
};

use model::ChatMessage;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 接收本次输入的字符串
    let mut input = String::new();
    // 聊天记录
    let mut chat_history = Vec::new();

    loop {
        terminal.draw(|f| ui(f, &input, &chat_history))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    // Send message
                    chat_history.push(ChatMessage {
                        sender: model::Sender::User,
                        message: input.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    });
                    input.clear();
                }
                KeyCode::Esc => {
                    // Exit the loop on `Esc`
                    break;
                }
                _ => {}
            }
        }
    }

    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    Ok(())
}

fn ui(f: &mut Frame, input: &str, chat_history: &[ChatMessage]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(80), Constraint::Length(3)].as_ref())
        .split(f.area());

    let chat_block = Block::default().title("Chat").borders(Borders::ALL);

    // Convert chat history into a list of ListItems
    let items: Vec<ListItem> = chat_history.iter().map(|m| ListItem::new(m.message.as_str())).collect();

    let chat_list = List::new(items)
        .block(chat_block)
        .style(Style::default().fg(Color::White));

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    let input_paragraph = Paragraph::new(input)
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(chat_list, chunks[0]);
    f.render_widget(input_paragraph, chunks[1]);
}
