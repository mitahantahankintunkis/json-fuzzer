use std::cmp::min;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode};

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Widget};

use crate::analyze::{Discrepancy, DosInfo};
use crate::server::TimeData;
use crate::util::{ns_to_string, TimeType};

pub struct ClientStatus {
    pub name: String,
    pub data: Option<TimeData>,
    pub parses_per_second: f64,
}

pub enum AppState {
    FocusTable,
    ConfirmQuit,
}

pub struct App {
    pub quit: bool,
    pub updated: bool,
    state: AppState,
    statuses: HashMap<String, ClientStatus>,
    last_seen: HashMap<String, Instant>,
    dos: Vec<DosInfo>,
    discrepancies: Vec<Discrepancy>,
    // terminal: DefaultTerminal,
}

impl App {
    pub fn new() -> Self {
        App {
            quit: false,
            updated: true,
            state: AppState::FocusTable,
            statuses: HashMap::new(),
            last_seen: HashMap::new(),
            dos: Vec::new(),
            discrepancies: Vec::new(),
            // terminal: ratatui::init(),
        }
    }

    pub fn handle_events(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(100)).unwrap() {
            return Ok(());
        }

        if let Some(key) = event::read()?.as_key_press_event() {
            self.updated = true;

            match self.state {
                AppState::FocusTable => {
                    match key.code {
                        // KeyCode::Char('y') | KeyCode::Esc => tui::confirm(),
                        KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::ConfirmQuit,
                        // KeyCode::Char('j') | KeyCode::Down => table_state.select_next(),
                        // KeyCode::Char('k') | KeyCode::Up => table_state.select_previous(),
                        // KeyCode::Char('l') | KeyCode::Right => table_state.select_next_column(),
                        // KeyCode::Char('h') | KeyCode::Left => table_state.select_previous_column(),
                        // KeyCode::Char('g') => table_state.select_first(),
                        // KeyCode::Char('G') => table_state.select_last(),
                        _ => {}
                    }
                }
                AppState::ConfirmQuit => {
                    match key.code {
                        KeyCode::Char('y') => self.quit = true,
                        // KeyCode::Char('q') | KeyCode::Esc => tui::quit(),
                        // KeyCode::Char('j') | KeyCode::Down => table_state.select_next(),
                        // KeyCode::Char('k') | KeyCode::Up => table_state.select_previous(),
                        // KeyCode::Char('l') | KeyCode::Right => table_state.select_next_column(),
                        // KeyCode::Char('h') | KeyCode::Left => table_state.select_previous_column(),
                        // KeyCode::Char('g') => table_state.select_first(),
                        // KeyCode::Char('G') => table_state.select_last(),
                        _ => self.state = AppState::FocusTable,
                    }
                }
            }
        }

        Ok(())
    }

    pub fn push_status(&mut self, status: ClientStatus) {
        self.last_seen.insert(status.name.clone(), Instant::now());
        self.statuses.insert(status.name.clone(), status);
        self.updated = true;
    }

    pub fn push_dos(&mut self, dos: DosInfo) {
        self.dos.push(dos);
        self.dos.sort_by(|a, b| b.ratio.total_cmp(&a.ratio));
        self.updated = true;
    }

    pub fn push_discrepancy(&mut self, discrepancy: Discrepancy) {
        self.discrepancies.push(discrepancy);
        if self.discrepancies.len() > 1000 {
            self.discrepancies.remove(0);
        }
        self.updated = true;
    }

    // fn render_statusline(s: &str, area: Rect, buf: &mut Buffer) {
    // }

    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        // let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(area);
        // let title_row = Line::from_iter([Span::from("asdf")]);
        // title_row.render(area, buf);

        let header = Row::new([
            "Client".into(),
            "TC".into(),
            Text::from("tn").alignment(ratatui::layout::Alignment::Right),
            Text::from("wn").alignment(ratatui::layout::Alignment::Right),
            Text::from("TC/s").alignment(ratatui::layout::Alignment::Right),
            Text::from("Prev").alignment(ratatui::layout::Alignment::Right),
        ])
        .style(Style::new().bold());

        let mut row_elems = self
            .statuses
            .values()
            .map(|status| {
                let (d, json, weight) = match &status.data {
                    Some(data) => {
                        let mut s = match ns_to_string(data.duration) {
                            (s, TimeType::Nanos) => s.into(),
                            (s, TimeType::Micros) => s.bold(),
                            (s, TimeType::Millis) => s.light_yellow().bold(),
                            (s, TimeType::Secs) => s.red().bold(),
                            (s, TimeType::Mins) => s.red().bold(),
                        };

                        if data.duration == 0.0 {
                            s = "...".into();
                        }

                        (
                            s,
                            data.testcase.json[0..min(data.testcase.json.len(), 40)].into(),
                            data.testcase.weight.to_string(),
                        )
                    }
                    None => ("".into(), "waiting...".dark_gray(), "".into()),
                };

                let dur = match self.last_seen.get(&status.name) {
                    Some(t) => Some(Instant::now().duration_since(*t).as_millis()),
                    None => None,
                };

                let dur = match dur {
                    Some(t @ 0..1000) => format!("{}ms", t).into(),
                    Some(t @ 1000..60_000) => format!("{}s", t / 1000).into(),
                    Some(t @ 60_000..3_600_000) => format!("{}m", t / 60_000).bold().light_yellow(),
                    Some(t @ 3_600_000..) => format!("{}h", t / 3_600_000).bold().red(),
                    None => "?".into(),
                };

                let per_s = match status.parses_per_second {
                    0.0 => format!("...").into(),
                    t @ ..1000.0 => format!("{:.0}", t).bold().red(),
                    t @ ..10_000.0 => format!("{:.2}k", t / 1000.0).bold().yellow(),
                    t @ ..100_000.0 => format!("{:.1}k", t / 1000.0).bold(),
                    t @ ..1000_000.0 => format!("{:.1}k", t / 1000.0).into(),
                    t @ _ => format!("{:.2}M", t / 1000_000.0).into(),
                };

                (
                    status.name.clone(),
                    [
                        status.name.clone().into(),
                        json,
                        // match json.len() {
                        //     0..40 => json.clone().into(),
                        //     _ => format!("{}...", &json[0..37]).into(),
                        // },
                        d,
                        format!("{:.2}", weight).into(),
                        // status.data.testcase.weight.to_string().into(),
                        per_s,
                        // "?".into(),
                        dur,
                    ],
                )
            })
            .collect::<Vec<(String, [Span; 6])>>();

        row_elems.sort_by(|a, b| a.0.cmp(&b.0));

        let min_widths = [6, 20, 6, 6, 6, 6];
        let mut widths: Vec<Constraint> = Vec::new(); //row_elems.iter().map(|r| Constraint::Length(r.len() as u16));
        if let Some(first_row) = row_elems.get(0) {
            for i in 0..first_row.1.len() {
                if i == 1 {
                    widths.push(Constraint::Fill(1));
                    continue;
                }

                let mut max = 0u16;

                for j in 0..row_elems.len() {
                    let len = row_elems[j].1[i].width() as u16;
                    if max < len {
                        max = len;
                    }
                }

                if max < min_widths[i] {
                    max = min_widths[i];
                }

                widths.push(Constraint::Length(max));
            }
        }

        let mut rows: Vec<Row> = Vec::new();

        for (_, r) in &row_elems {
            // rows.push(Row::from_iter(r.clone()));
            // rows.push(Row::default().cells(r.as_ref().iter().map(|c| Cell::new(c.clone()))));
            rows.push(
                r.as_ref()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        if i < 2 {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Left),
                            )
                        } else {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Right),
                            )
                        }
                    })
                    .collect::<Row>(),
            );

            // let mut cells = Vec::new();
            // for i in 0..r.len() {
            //     // if i >= 2 {
            //     //     let text =
            //     //         Text::from(r[i].clone()).alignment(ratatui::layout::Alignment::Right);
            //     //     cells.push(Cell::new(text));
            //     // } else {
            //     cells.push(Cell::new(r[i].clone()))
            //     // }
            // }
            //
            // rows.push(Row::default().cells(cells));
        }

        // let rows = [
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        //     Row::new(["c_mjson", "{\"q\":42}", "420ns", "1.23", "3.4M/s"]),
        // ];

        // let footer = Row::new([
        //     "Ratatouille Recipe",
        //     "",
        //     "135 kcal, 31g carbs, 6.4g protein",
        // ]);

        // let widths = [
        //     Constraint::Percentage(30),
        //     Constraint::Percentage(20),
        //     Constraint::Percentage(50),
        // ];

        let block = Block::new()
            .title("Scheduler status")
            .borders(Borders::all());
        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            // .footer(footer.italic())
            .column_spacing(1)
            .style(Color::White);
        // .row_highlight_style(Style::new().on_black().bold())
        // .column_highlight_style(Color::Gray)
        // .cell_highlight_style(Style::new().reversed().yellow())
        // .highlight_symbol("🍴 ");

        table.render(area, buf);
    }

    fn render_sidebar_dos(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new([
            "Client".into(),
            "TC".into(),
            Text::from("tn").alignment(ratatui::layout::Alignment::Right),
            Text::from("rn").alignment(ratatui::layout::Alignment::Right),
            Text::from("t/b").alignment(ratatui::layout::Alignment::Right),
        ])
        .style(Style::new().bold());

        let row_elems = self
            .dos
            .iter()
            .map(|dos| {
                [
                    dos.parser.clone().into(),
                    dos.testcase.json.clone().into(),
                    match ns_to_string(dos.parsing_time) {
                        (s, TimeType::Nanos) => s.into(),
                        (s, TimeType::Micros) => s.bold(),
                        (s, TimeType::Millis) => s.light_yellow().bold(),
                        (s, TimeType::Secs) => s.red().bold(),
                        (s, TimeType::Mins) => s.red().bold(),
                    },
                    match dos.ratio {
                        ..1.0 => format!("{:.2}", dos.ratio).gray(),
                        1.0..1.5 => format!("{:.2}", dos.ratio).into(),
                        1.5..2.0 => format!("{:.2}", dos.ratio).bold(),
                        2.0.. => format!("{:.2}", dos.ratio).light_yellow().bold(),
                        _ => "?".into(),
                    },
                    // format!("{:}", dos.ns_per_ch).into(),
                    match ns_to_string(dos.ns_per_ch) {
                        (s, TimeType::Nanos) => s.into(),
                        (s, TimeType::Micros) => s.bold(),
                        (s, TimeType::Millis) => s.light_yellow().bold(),
                        (s, TimeType::Secs) => s.red().bold(),
                        (s, TimeType::Mins) => s.red().bold(),
                    },
                ]
            })
            .collect::<Vec<[Span; 5]>>();

        let mut widths: Vec<Constraint> = Vec::new();
        if let Some(first_row) = row_elems.get(0) {
            for i in 0..first_row.len() {
                if i == 1 {
                    widths.push(Constraint::Fill(1));
                    continue;
                }

                let mut max = 1u16;

                for j in 0..row_elems.len() {
                    let len = row_elems[j][i].width() as u16;
                    if max < len {
                        max = len;
                    }
                }

                widths.push(Constraint::Length(max + 1));
            }
        }

        let mut rows: Vec<Row> = Vec::new();

        for r in &row_elems {
            rows.push(
                r.as_ref()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        if i < 2 {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Left),
                            )
                        } else {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Right),
                            )
                        }
                    })
                    .collect::<Row>(),
            );
        }

        let block = Block::new()
            .title("Bug oracle: Denial of Service")
            .borders(Borders::all());
        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .column_spacing(1)
            .style(Color::White);

        table.render(area, buf);
    }

    fn render_sidebar_discrepancies(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new([
            "Parser0".into(),
            "Parser1".into(),
            "TC".into(),
            Text::from("out0").alignment(ratatui::layout::Alignment::Right),
            Text::from("out1").alignment(ratatui::layout::Alignment::Right),
        ])
        .style(Style::new().bold());

        let row_elems = self
            .discrepancies
            .iter()
            .rev()
            .take(40)
            .map(|discrepancy| {
                [
                    discrepancy.parser_0_name.clone().into(),
                    discrepancy.parser_1_name.clone().into(),
                    discrepancy.testcase.json.clone().into(),
                    discrepancy
                        .parser_0_output
                        .replace("\t", "\\t")
                        .replace("\n", "\\n")
                        .replace("\r", "\\r")
                        .into(),
                    discrepancy
                        .parser_1_output
                        .replace("\t", "\\t")
                        .replace("\n", "\\n")
                        .replace("\r", "\\r")
                        .into(),
                ]
            })
            .collect::<Vec<[Span; 5]>>();

        let mut widths: Vec<Constraint> = Vec::new();
        if let Some(first_row) = row_elems.get(0) {
            for i in 0..first_row.len() {
                if i == 2 {
                    widths.push(Constraint::Fill(1));
                    continue;
                }

                let mut max = 4u16;

                for j in 0..row_elems.len() {
                    let len = row_elems[j][i].width() as u16;
                    if max < len {
                        max = len;
                    }
                }

                widths.push(Constraint::Length(max + 1));
            }
        }

        let mut rows: Vec<Row> = Vec::new();

        for r in &row_elems {
            rows.push(
                r.as_ref()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        if i < 3 {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Left),
                            )
                        } else {
                            Cell::from(
                                Text::from(c.clone()).alignment(ratatui::layout::Alignment::Right),
                            )
                        }
                    })
                    .collect::<Row>(),
            );
        }

        let block = Block::new()
            .title("Bug oracle: Discrepancies")
            .borders(Borders::all());
        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .column_spacing(1)
            .style(Color::White);

        table.render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
        let content_cont = layout[0];
        let statusline_cont = layout[1];

        let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Min(1)])
            .split(content_cont);

        let table_cont = layout[0];
        let sidebar_cont = layout[1];

        let sidebar_layout =
            Layout::vertical([Constraint::Percentage(50), Constraint::Min(1)]).split(sidebar_cont);
        let discrepancy_cont = sidebar_layout[0];
        let dos_cont = sidebar_layout[1];

        self.render_table(table_cont, buf);
        self.render_sidebar_dos(dos_cont, buf);
        self.render_sidebar_discrepancies(discrepancy_cont, buf);

        match self.state {
            AppState::ConfirmQuit => {
                Line::from_iter(["Quit".bold(), "(y/n)".bold().light_yellow(), "?".bold()])
                    // .centered()
                    .render(statusline_cont, buf);
            }
            _ => {}
        }
        // Line::from_iter(["Quit".bold(), "(y/n)".bold().light_yellow(), "?".bold()])
        //     .render(statusline_cont, buf);

        // MyHeaderWidget::new("Header text").render(Rect::new(0, 0, area.width, 1), buf);
        // frame.render_widget("hello world", frame.area());
    }
}
