use ratatui::{
    self, Frame,
    crossterm::event::{
        self,
        Event::{self},
        KeyCode,
    },
    layout::{
        Constraint::{self},
        Direction, Layout,
    },
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::structs::{App, ProxyState};

pub async fn run_tui(
    state: Arc<Mutex<ProxyState>>,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut term = ratatui::init();

    let mut table_state = TableState::default();

    loop {
        {
            let log_manager = &mut app.log_m;
            while let Ok(log) = log_manager.rx.try_recv() {
                log_manager.logs.push_back(log);
                if log_manager.logs.len() > 10000 {
                    log_manager.logs.pop_front();
                }
            }
        }

        let current = table_state.selected().unwrap_or(0);
        term.draw(|f| render(f, &app, table_state))?;

        let filters = &mut app.inputs;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    if c == 'x' {
                        app.filtered_funcs_vec.clear();
                        let mut guard = state.lock().await;
                        guard.filtered_funcs.iter_mut().for_each(|x| *x = false);
                        guard.is_func_filter = false;
                    } else if c == 's' {
                        app.autoscroll = !app.autoscroll;
                    } else {
                        filters.func_code_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    filters.func_code_input.pop();
                }
                KeyCode::Enter => {
                    let func_num = filters.func_code_input.parse::<u8>().unwrap_or_default();
                    {
                        let mut guard = state.lock().await;

                        guard.is_func_filter = true;
                        guard.filtered_funcs[func_num as usize] = true;
                    }

                    app.filtered_funcs_vec.push(func_num);
                }
                KeyCode::Down => {
                    if current < app.log_m.logs.len() - 1 {
                        table_state.select(Some(current + 1));
                    }
                }
                KeyCode::Up if current > 0 => {
                    table_state.select(Some(current - 1));
                }
                _ => {}
            }
        }
    }
    ratatui::restore();
    Ok(())
}

fn render(frame: &mut Frame, app: &App, mut table_state: TableState) {
    let logs = &app.log_m.logs;
    if app.autoscroll {
        table_state.select(Some(logs.len() - 1));
    }
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    let left = chunks[0];

    let right = chunks[1];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right);

    let right_up = chunks[0];

    let right_down = chunks[1];
    let table = Table::from_iter((0..logs.len()).map(|index| {
        let log = &logs[index];

        Row::new(vec![
            format!("{}", index + 1),
            format!("{}", log.header.transaction_id),
            format!("{}", log.header.protocol_id),
            format!("{}", log.header.length),
            format!("0x{:02X}", log.header.unit_id),
            log.func_code.to_string(),
            log.peer.to_string(),
        ])
    }))
    .widths([
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(14),
    ])
    .header(Row::new(vec![
        "idx",
        "TX ID",
        "PROTO ID",
        "LEN",
        "UNIT ID",
        "FUNC",
        "PEER IP ADDR",
    ]))
    .highlight_symbol(">>")
    .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    let input_func_code = Paragraph::new(app.inputs.func_code_input.as_str())
        .block(Block::default().borders(Borders::ALL).title("filter func"));

    let filters_status = Paragraph::new(format!("FUNC FILTERED: {:?}", app.filtered_funcs_vec))
        .wrap(Wrap { trim: true });
    frame.render_widget(filters_status, right_down);
    frame.render_widget(input_func_code, right_up);
    frame.render_stateful_widget(table, left, &mut table_state);
}
