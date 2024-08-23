use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_key(frame: &mut Frame, key: &str, key_index: u16) {
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
