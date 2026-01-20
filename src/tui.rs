#[cfg(feature = "tui")]
use std::io;
#[cfg(feature = "tui")]
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "tui")]
use ratatui::{prelude::*, widgets::*};

#[cfg(feature = "tui")]
pub fn run_tui() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut input = String::from("192.168.1.0/24");

    loop {
        terminal.draw(|f| ui(f, &input))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => { input.pop(); },
                KeyCode::Enter => { /* Could trigger specific action */ }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(feature = "tui")]
fn ui(f: &mut Frame, input: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(10),   // Results
            Constraint::Length(3), // Help
        ])
        .split(f.size());

    // 1. Input Box
    let input_panel = Paragraph::new(input)
        .block(Block::default().borders(Borders::ALL).title(" Enter CIDR (e.g. 10.0.0.1/24) "));
    f.render_widget(input_panel, chunks[0]);

    // 2. Logic Integration (Connecting to your existing calc logic)
    // Assuming your lib has a function like `calculate(input: &str)`
    let display_text = if let Ok(net) = input.parse::<ipnet::IpNet>() {
        format!(
            "Network:    {}\nMask:       {}\nBroadcast:  {}\nFirst Host: {}\nLast Host:  {}",
            net.network(),
            net.netmask(),
            net.broadcast(),
            net.hosts().next().unwrap_or(net.network()),
            net.hosts().last().unwrap_or(net.network())
        )
    } else {
        "Invalid IP/CIDR".to_string()
    };

    let results = Paragraph::new(display_text)
        .block(Block::default().borders(Borders::ALL).title(" Calculations "))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(results, chunks[1]);

    // 3. Help Bar
    let help = Paragraph::new(" ESC to Quit | Type to update real-time ")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}