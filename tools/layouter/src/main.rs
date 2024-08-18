#![warn(clippy::all, clippy::pedantic)]
mod keyboard;

use std::collections::HashMap;
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{Event, EventStream, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style::Stylize,
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io::{stdout, Result};
use tokio::{
    sync::mpsc,
    task,
    time::{self, Duration},
};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    let keyboard_task = task::spawn(async move {
        let mut keyboard = keyboard::Keyboard::new().expect("Could not open the keyboard");
        let mut current_layer = keyboard::Layer::Base;
        let mut interval = time::interval(Duration::from_millis(16));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match keyboard.get_current_layer() {
                        Ok(layer) => {
                            if layer == current_layer {
                                continue;
                            }

                            if tx.send(layer).await.is_err() {
                                println!("Could not send layer change to the main task");
                                break;
                            }

                            current_layer = layer;
                        }
                        Err(err) => {
                            println!("Error reading layer change: {err:?}");
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => break,
            }
        }
    });

    let mut layers: HashMap<keyboard::Layer,[&str; 42]> = HashMap::new();

    layers.insert(keyboard::Layer::Base, [
        "tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "<-", "ctrl", "A", "S", "D",
        "F", "G", "H", "J", "K", "L", ";", "\"", "shift", "Z", "X", "C", "V", "B", "N", "M",
        ",", ".", "/", "esc", "cmd", "MO(1)", "--", "\u{23CE}", "MO(2)", "alt",
    ]);

    layers.insert(keyboard::Layer::BaseShift, [
        "tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "<-", "ctrl", "A", "S", "D",
        "F", "G", "H", "J", "K", "L", ";", "\"", "shift", "Z", "X", "C", "V", "B", "N", "M",
        ",", ".", "/", "esc", "cmd", "MO(1)", "--", "\u{23CE}", "MO(2)", "alt",
    ]);

    layers.insert(keyboard::Layer::Lower, [
        "tab", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "<-", "ctrl", "", "", "", "",
        "", "LEFT", "DOWN", "UP", "RIGHT", "", "", "shift", "", "", "", "", "", "", "", "", "",
        "", "", "cmd", "__", "--", "\u{23CE}", "MO(3)", "alt",
    ]);

    layers.insert(keyboard::Layer::LowerShift, [
        "tab", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "<-", "ctrl", "", "", "", "",
        "", "LEFT", "DOWN", "UP", "RIGHT", "", "", "shift", "", "", "", "", "", "", "", "", "",
        "", "", "cmd", "__", "--", "\u{23CE}", "MO(3)", "alt",
    ]);

    layers.insert(keyboard::Layer::Raise, [
        "tab", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "<-", "ctrl", "", "", "", "",
        "", "-", "=", "[", "]", "\\", "`", "shift", "", "", "", "", "", "_", "+", "{", "}",
        "|", "~", "cmd", "MO(3)", "--", "\u{23CE}", "__", "alt",
    ]);

    layers.insert(keyboard::Layer::RaiseShift, [
        "tab", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "<-", "ctrl", "", "", "", "",
        "", "-", "=", "[", "]", "\\", "`", "shift", "", "", "", "", "", "_", "+", "{", "}",
        "|", "~", "cmd", "MO(3)", "--", "\u{23CE}", "__", "alt",
    ]);

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut interval = time::interval(Duration::from_millis(16));
    let mut current_layer = keyboard::Layer::Base;

    let mut events = EventStream::new();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                terminal.draw(|frame| {
                    let keys = layers.get(&current_layer).expect("Could not find the current layer");

                    for (i, key) in keys.iter().enumerate() {
                      draw_key(frame, key, u16::try_from(i).expect("Could not convert usize to u16"));
                    }
                })?;
            }
            Some(layer) = rx.recv() => { current_layer = layer}

            Some(event) = events.next().fuse() => {
                if let Ok(Event::Key(key)) = event {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        shutdown_tx.send(()).await.expect("Could not send shutdown signal");
                        break;
                    }
                }
            }
        }
    }

    keyboard_task.await?;

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// Draw a key on the frame at the given position, with offsets
fn draw_key(frame: &mut Frame, key: &str, key_index: u16) {
    const INITIAL_OFFSET_Y: u16 = 3;

    const KEY_WIDTH: u16 = 8;
    const KEY_HEIGHT: u16 = 4;
    const KEY_SPACING: u16 = 1;
    const KEY_ROWS: u16 = 12;

    let column = key_index % KEY_ROWS;
    let row = key_index / KEY_ROWS;

    #[allow(clippy::match_same_arms)]
    let mut column_offset = match column {
        2 => 2,
        3 => 3,
        4 => 2,
        5 => 1,
        6 => 1,
        7 => 2,
        8 => 3,
        9 => 2,
        _ => 0,
    };

    let mut row_offset = match column {
        6..12 => 10,
        _ => 0,
    };

    if row == 3 {
        row_offset += 27;
        column_offset = 0;

        if column > 2 {
            row_offset += 10;
        }
    }

    let x = column * (KEY_WIDTH + KEY_SPACING) + row_offset;
    let y = row * (KEY_HEIGHT + KEY_SPACING) + INITIAL_OFFSET_Y - column_offset;

    let rect = Rect {
        x,
        y,
        width: KEY_WIDTH,
        height: KEY_HEIGHT,
    };

    let key_widget = Paragraph::new(key)
        .style(Style::default().fg(Color::White).bg(Color::Black).bold())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .centered();

    frame.render_widget(key_widget, rect);
}
