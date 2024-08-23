#![warn(clippy::all, clippy::pedantic)]
mod keyboard;
mod ui;

use futures::{FutureExt, StreamExt};
use hidapi::HidError;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{Event, EventStream, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};
use std::collections::HashMap;
use std::io::stdout;
use tokio::{
    sync::mpsc,
    task,
    time::{self, Duration},
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let (tx, mut rx) = mpsc::channel(32);

    let token = CancellationToken::new();

    let keyboard_shutdown = token.clone();

    let keyboard_task = task::spawn(async move {
        let mut keyboard = keyboard::Keyboard::new().map_err(|err| {
            keyboard_shutdown.cancel();
            err
        })?;

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

                            tx.send(layer).await.map_err(|err| {
                                keyboard_shutdown.cancel();
                                HidError::IoError { error: std::io::Error::new(std::io::ErrorKind::BrokenPipe, err) }
                            })?;

                            current_layer = layer;
                        }
                        Err(err) => return Err(err),
                    }
                }
                _ = keyboard_shutdown.cancelled() => break,
            }
        }

        Ok(())
    });

    let mut layers: HashMap<keyboard::Layer, [&str; 42]> = HashMap::new();

    layers.insert(
        keyboard::Layer::Base,
        [
            "tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "<-", "ctrl", "A", "S", "D",
            "F", "G", "H", "J", "K", "L", ";", "\"", "shift", "Z", "X", "C", "V", "B", "N", "M",
            ",", ".", "/", "esc", "cmd", "MO(1)", "--", "\u{23CE}", "MO(2)", "alt",
        ],
    );

    layers.insert(
        keyboard::Layer::BaseShift,
        [
            "tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "<-", "ctrl", "A", "S", "D",
            "F", "G", "H", "J", "K", "L", ";", "\"", "shift", "Z", "X", "C", "V", "B", "N", "M",
            ",", ".", "/", "esc", "cmd", "MO(1)", "--", "\u{23CE}", "MO(2)", "alt",
        ],
    );

    layers.insert(
        keyboard::Layer::Lower,
        [
            "tab", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "<-", "ctrl", "", "", "", "",
            "", "LEFT", "DOWN", "UP", "RIGHT", "", "", "shift", "", "", "", "", "", "", "", "", "",
            "", "", "cmd", "__", "--", "\u{23CE}", "MO(3)", "alt",
        ],
    );

    layers.insert(
        keyboard::Layer::LowerShift,
        [
            "tab", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "<-", "ctrl", "", "", "", "",
            "", "LEFT", "DOWN", "UP", "RIGHT", "", "", "shift", "", "", "", "", "", "", "", "", "",
            "", "", "cmd", "__", "--", "\u{23CE}", "MO(3)", "alt",
        ],
    );

    layers.insert(
        keyboard::Layer::Raise,
        [
            "tab", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "<-", "ctrl", "", "", "", "",
            "", "-", "=", "[", "]", "\\", "`", "shift", "", "", "", "", "", "_", "+", "{", "}",
            "|", "~", "cmd", "MO(3)", "--", "\u{23CE}", "__", "alt",
        ],
    );

    layers.insert(
        keyboard::Layer::RaiseShift,
        [
            "tab", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "<-", "ctrl", "", "", "", "",
            "", "-", "=", "[", "]", "\\", "`", "shift", "", "", "", "", "", "_", "+", "{", "}",
            "|", "~", "cmd", "MO(3)", "--", "\u{23CE}", "__", "alt",
        ],
    );

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
                      ui::draw_key(frame, key, u16::try_from(i).expect("Could not convert usize to u16"));
                    }
                })?;
            }
            Some(layer) = rx.recv() => { current_layer = layer}

            Some(event) = events.next().fuse() => {
                if let Ok(Event::Key(key)) = event {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        token.cancel();
                        break;
                    }
                }
            }

           () = token.cancelled() => break,
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    match keyboard_task.await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => eprintln!("Error: {err}"),
        Err(err) => eprintln!("Error: {err}"),
    }

    Ok(())
}
