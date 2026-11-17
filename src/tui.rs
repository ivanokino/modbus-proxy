use crate::structs::{App, ProxyState};

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
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    thread,
};
use tokio::sync::Mutex;

pub async fn run_tui(
    state: Arc<Mutex<ProxyState>>,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut term = ratatui::init();

    let table_state = TableState::default();

    let (tx_key, mut rx_key) = tokio::sync::mpsc::channel(100);
    let _ = thread::spawn(move || {
        loop {
            if let Event::Key(key) = match event::read() {
                Ok(x) => x,
                Err(_) => continue,
            } {
                let _ = tx_key.blocking_send(key.code);
            }
        }
    });

    let mut current = 0;
    let _ = term.draw(|f| render(f, &app, table_state, 0));

    'main_gui_loop: loop {
        let log_manager = &mut app.log_m;

        tokio::select! {

            //// 1 - IF KEY PRESSED
            Some(key) = rx_key.recv() => {

                let filters = &mut app.inputs;
                match key {
                    KeyCode::Esc => break 'main_gui_loop,
                    KeyCode::Tab => app.input_select = !app.input_select,
                    KeyCode::Char(c) => match c {
                        'x' => {
                            if !app.input_select {
                                app.filtered_funcs_vec.clear();
                                let mut guard = state.lock().await;
                                guard.filtered_funcs.iter_mut().for_each(|x| *x = false);
                                guard.is_func_filter = false;
                            } else {
                                app.filtered_ip_vec.clear();
                                let mut guard = state.lock().await;
                                guard.filtered_ip_hashset.clear();
                                guard.is_ip_filter = false;
                            }
                        }
                        's' => {
                            app.autoscroll = !app.autoscroll;
                        }

                        _ => {
                            if !app.input_select {
                                filters.func_code_input.push(c);
                            } else {
                                filters.ip_addr_input.push(c);
                            }
                        }
                    },
                    KeyCode::Backspace => {
                        if !app.input_select {
                            filters.func_code_input.pop();
                        } else {
                            filters.ip_addr_input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        if !app.input_select {
                            let func_num = filters.func_code_input.parse::<u8>().unwrap_or_default();
                            {
                                let mut guard = state.lock().await;

                                guard.is_func_filter = true;
                                guard.filtered_funcs[func_num as usize] = true;
                                app.filtered_funcs_vec.push(func_num);
                            }
                        } else {
                            let ip = match filters.ip_addr_input.parse::<IpAddr>() {
                                Ok(ip) => ip,
                                Err(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                            };

                            {
                                let mut guard = state.lock().await;
                                guard.is_ip_filter = true;
                                guard.filtered_ip_hashset.insert(ip);
                                app.filtered_ip_vec.push(ip);
                            }
                        }
                    }

                    KeyCode::Down => {
                        if current < app.log_m.logs.len().saturating_sub(1) {
                            current +=1;
                        }
                    }
                    KeyCode::Up if current > 0 => {
                        current -= 1;
                    }
                    _ => {}
                }
                let _ = term.draw(|f| render(f, &app, table_state, current));
            },


            //// 2 - IF LOG RECEIVED
            Some(log) = log_manager.rx.recv() => {
                log_manager.logs.push_back(log);
                app.logs_total +=1;
                if log_manager.logs.len() > 100000 {
                    log_manager.logs.pop_front();
                }
                let _ = term.draw(|f| render(f, &app, table_state, current));
            },

        }
    }
    ratatui::restore();
    Ok(())
}

fn render(frame: &mut Frame, app: &App, mut table_state: TableState, mut table_current: usize) {
    let logs = &app.log_m.logs;
    if app.autoscroll {
        table_current = logs.len().saturating_sub(1);
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_up);

    let right_up_up = chunks[0];
    let right_up_down = chunks[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_down);

    let right_down_left = chunks[0];
    let right_down_right = chunks[1];

    let visible = frame.area().height.saturating_sub(1) as usize; // how much rows in a frame 
    let top = (table_current + 1).saturating_sub(visible); // uppest row
    // table_current index relative to the frame (not vec)
    table_state.select(Some(table_current.saturating_sub(top)));

    let table = Table::from_iter((top..(top + visible).min(logs.len())).map(|index| {
        let log = &logs[index];

        Row::new(vec![
            ((app.logs_total as usize - logs.len() + 1 + index) as u64).to_string(),
            log.header.transaction_id.to_string(),
            log.header.protocol_id.to_string(),
            log.header.length.to_string(),
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
        Constraint::Length(22),
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
    let input_ip_addr = Paragraph::new(app.inputs.ip_addr_input.as_str())
        .block(Block::default().borders(Borders::ALL).title("filter IP"));

    let filter_func_status = Paragraph::new(format!("FUNC FILTERED: {:?}", app.filtered_funcs_vec))
        .wrap(Wrap { trim: true });

    let filter_ip_status =
        Paragraph::new(format!("iP FILTERED: {:?}", app.filtered_ip_vec)).wrap(Wrap { trim: true });

    frame.render_widget(filter_func_status, right_down_left);
    frame.render_widget(filter_ip_status, right_down_right);
    frame.render_widget(input_func_code, right_up_up);
    frame.render_widget(input_ip_addr, right_up_down);
    frame.render_stateful_widget(table, left, &mut table_state);
}
