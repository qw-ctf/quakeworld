#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

trait AssignIfGreater {
    fn assign_if_greater(&mut self, size: usize);
}

impl AssignIfGreater for usize {
    fn assign_if_greater(&mut self, size: usize) {
        if *self < size {
            *self = size;
        }
    }
}
pub mod args;
pub mod tracelist;
mod utils;

use clap::Parser;

use args::TraceviewerArgs;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::stdout;
use std::io::Read;
use std::time::Instant;

use quakeworld::mvd::Mvd;
use quakeworld::state::State;

mod pak;
use pak::trace_pak;

mod mvd;
use mvd::trace_mvd;

mod bsp;
use bsp::trace_bsp;

use color_eyre::{config::HookBuilder, owo_colors::OwoColorize};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use quakeworld::trace::{Trace, TraceEntry};
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use tracelist::Tracelist;

const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const SELECTED_TEXT_COLOR: Color = tailwind::SLATE.c900;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;
const HEX_COLORS: [Color; 4] = [
    tailwind::GREEN.c600,
    tailwind::ROSE.c500,
    tailwind::CYAN.c500,
    tailwind::SLATE.c500,
];

#[derive(Copy, Clone)]
enum UiLayout {
    ListLeft,
    ListRight,
}

#[derive(Default)]
struct Debug {
    initialization: HashMap<String, DebugValue>,
    tracelist: HashMap<String, DebugValue>,
    subtraces: HashMap<String, DebugValue>,
    enabled: bool,
}

impl Debug {
    pub fn tracelist_add(&mut self, name: impl Into<String>, value: impl std::fmt::Debug) {
        let name = name.into();
        let value = format!("{:?}", value);
        self.tracelist.insert(name, DebugValue::String(value));
    }
    pub fn subtraces_add(&mut self, name: impl Into<String>, value: impl std::fmt::Debug) {
        let name = name.into();
        let value = format!("{:?}", value);
        self.subtraces.insert(name, DebugValue::String(value));
    }
}

struct HexState {
    width: u64,
}

struct TraceReplace {
    pub trace: TraceEntry,
    pub enabled: bool,
}

struct App<'a> {
    layout: UiLayout,
    data: Vec<u8>,
    trace: TraceReplace,
    tracelist_read: Tracelist<'a>,
    tracelist_stack: Tracelist<'a>,
    debug: Debug,
    hex: HexState,
    value_enabled: bool,
    show_help: bool,
    show_stack_traces: bool,
    selected_trace: Option<&'a TraceEntry>,
}

fn read_file(path: std::path::PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(Box::new(err)),
    };
    match file.read_to_end(&mut buffer) {
        Ok(size) => size,
        Err(err) => return Err(Box::new(err)),
    };
    return Ok(buffer);
}
#[derive(Debug)]
struct QtvError(String);

impl fmt::Display for QtvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for QtvError {}

fn trace_qtv(options: args::TraceCommandQtv) -> Result<TraceView, Box<dyn Error>> {
    let _data = read_file(options.file)?;
    Err(Box::new(QtvError("not yet implemented :P".into())))
}

struct TraceView {
    pub data: Vec<u8>,
    pub read_trace: TraceReplace,
    pub trace_entry_list_read: TraceEntry,
    pub trace_entry_list_stack: TraceEntry,
    pub initialization_traces: HashMap<String, DebugValue>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = TraceviewerArgs::parse();
    let _ret = match args.command {
        args::CommandType::Trace(options) => {
            let traces = match options {
                args::TraceCommandType::Mvd(options) => trace_mvd(options),
                args::TraceCommandType::Qtv(options) => trace_qtv(options),
                args::TraceCommandType::Pak(options) => trace_pak(options),
                args::TraceCommandType::Bsp(options) => trace_bsp(options),
            };
            match traces {
                Ok(t) => {
                    init_error_hooks()?;
                    let terminal = init_terminal()?;
                    App::new(
                        t.data,
                        t.read_trace,
                        t.trace_entry_list_read,
                        t.trace_entry_list_stack,
                        t.initialization_traces,
                    )
                    .run(terminal)?;

                    restore_terminal()?;
                }
                Err(e) => {
                    println!("{}", e);
                    return Err(e);
                }
            }
        }
    };
    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

impl<'a> App<'a> {
    fn new(
        data: Vec<u8>,
        trace: TraceReplace,
        tracelist_read: TraceEntry,
        tracelist_stack: TraceEntry,
        initialization_traces: HashMap<String, DebugValue>,
    ) -> Box<App<'a>> {
        Box::new(Self {
            data,
            trace,
            layout: UiLayout::ListLeft,
            tracelist_read: Tracelist::new(tracelist_read),
            tracelist_stack: Tracelist::new(tracelist_stack),
            debug: Debug {
                enabled: false,
                initialization: initialization_traces,
                ..Default::default()
            },
            hex: HexState { width: 32 },
            value_enabled: false,
            show_help: false,
            show_stack_traces: false,
            selected_trace: None,
        })
    }
}

impl<'a> App<'a> {
    fn trace_get_current_selected(&mut self) -> Option<&TraceEntry> {
        if self.show_stack_traces {
            self.tracelist_stack.trace_get_current_selected()
        } else {
            self.tracelist_read.trace_get_current_selected()
        }
    }

    // fn trace_set_current_selected(&mut self) {
    //     if self.show_stack_traces {
    //         self.tracelist_stack.trace_get_current_selected()
    //     } else {
    //         self.tracelist_read.trace_get_current_selected()
    //     }
    // }

    fn index_current(&self) -> usize {
        if self.show_stack_traces {
            self.tracelist_stack.index_current()
        } else {
            self.tracelist_read.index_current()
        }
    }

    fn trace_get_current_highlighted(&self) -> Option<&TraceEntry> {
        if self.show_stack_traces {
            self.tracelist_stack.trace_get_current_highlighted()
        } else {
            self.tracelist_read.trace_get_current_highlighted()
        }
    }

    fn trace_get_with_offset(&self, height: usize) -> (usize, usize) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_get_with_offset(height)
        } else {
            self.tracelist_read.trace_get_with_offset(height)
        }
    }

    fn trace_set_last_selected(&mut self, index_offset: usize) {
        if self.show_stack_traces {
            self.tracelist_stack.set_last_selected(index_offset)
        } else {
            self.tracelist_read.set_last_selected(index_offset)
        }
    }

    fn get_last_selected(&self) -> usize {
        if self.show_stack_traces {
            self.tracelist_stack.get_last_selected()
        } else {
            self.tracelist_read.get_last_selected()
        }
    }

    fn trace_move(&mut self, arg: i32) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_move(arg)
        } else {
            self.tracelist_read.trace_move(arg)
        }
    }

    fn trace_top(&mut self) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_top()
        } else {
            self.tracelist_read.trace_top()
        }
    }

    fn trace_bottom(&mut self) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_bottom()
        } else {
            self.tracelist_read.trace_bottom()
        }
    }

    fn trace_enter(&mut self) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_enter()
        } else {
            self.tracelist_read.trace_enter()
        }
    }

    fn trace_leave(&mut self) {
        if self.show_stack_traces {
            self.tracelist_stack.trace_leave()
        } else {
            self.tracelist_read.trace_leave()
        }
    }
}

impl App<'_> {
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('j') => self.trace_move(1),
                        Char('J') => self.trace_move(10),
                        Char('u') => self.trace_move(100),
                        Char('U') => self.trace_move(1000),
                        Char('k') => self.trace_move(-1),
                        Char('K') => self.trace_move(-10),
                        Char('i') => self.trace_move(-100),
                        Char('I') => self.trace_move(-1000),
                        Char('g') => self.trace_top(),
                        Char('G') => self.trace_bottom(),
                        Char('l') => self.trace_enter(),
                        Char('h') => self.trace_leave(),
                        Char('1') => {
                            self.layout = match self.layout {
                                UiLayout::ListLeft => UiLayout::ListRight,
                                UiLayout::ListRight => UiLayout::ListLeft,
                            }
                        }
                        Char('2') => self.debug.enabled = !self.debug.enabled,
                        Char('3') => self.value_enabled = !self.value_enabled,
                        Char('4') => self.show_stack_traces = !self.show_stack_traces,
                        F(1) => self.show_help = !self.show_help,
                        _ => {}
                    }
                }
            }
        }
    }
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.show_help {
            let mut s: Vec<_> = vec![format!("Keybindings:")];
            s.push(format!("q, Esc - quit"));
            s.push(format!("h/l - leave/enter trace"));
            s.push(format!("j/k - down up by 1"));
            s.push(format!("J/K - down up by 10"));
            s.push(format!("u/i - down up by 100"));
            s.push(format!("U/I - down up by 1000"));
            s.push(format!("g/G - go to the top/bottom of the current trace"));
            s.push(format!("1 - toggle hexview left/right"));
            s.push(format!("2 - toggle debug display"));
            s.push(format!("3 - toggle value display"));
            s.push(format!("4 - toggle finished/unfinished trace display"));
            s.push(format!("F1 - open/close help"));

            Paragraph::new(s.join("\n")).centered().render(area, buf);
            return;
        }
        let sides = match self.layout {
            UiLayout::ListLeft => (40, 60),
            UiLayout::ListRight => (60, 40),
        };
        let horizontal = Layout::horizontal([
            Constraint::Percentage(sides.0),
            Constraint::Percentage(sides.1),
        ]);
        let [left, right] = horizontal.areas(area);
        // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]);
        let side = match self.layout {
            UiLayout::ListLeft => (left, right),
            UiLayout::ListRight => (right, left),
        };
        let [rest_area, footer_area] = vertical.areas(side.0);

        // Create two chunks with equal vertical screen space. One for the list and the other for
        // the info block.
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [upper_item_list_area, lower_item_list_area] = vertical.areas(rest_area);

        self.render_tracelist_v2(upper_item_list_area, buf);
        self.render_subtraces(lower_item_list_area, buf);
        if self.debug.enabled {
            self.render_debug(side.1, buf);
        } else {
            if self.value_enabled {
                self.render_value(side.1, buf);
            } else {
                self.render_hex(side.1, buf);
            }
        }
        render_footer(footer_area, buf);
    }
}

trait AsItem {
    fn to_list_item(self, index: usize, last_selected: usize) -> ListItem<'static>;
}

impl AsItem for TraceEntry {
    fn to_list_item(self, index: usize, last_selected: usize) -> ListItem<'static> {
        let mut style = Style::new();
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        style.bg = Some(bg_color);
        style.fg = Some(TEXT_COLOR);
        if last_selected == index {
            style.bg = Some(SELECTED_STYLE_FG);
            style.fg = Some(SELECTED_TEXT_COLOR);
        }

        let line = Line::styled(
            format!(
                "{} {} - {} {}",
                self.field_type, self.field_name, self.index, self.index_stop
            ),
            style,
        );
        ListItem::new(line).bg(bg_color)
    }
}

trait StyleForTrace {
    fn get_style(&self, index: u64) -> Style;
    fn get_style_with_bg(&self, index: u64, bg: Option<Color>) -> Style;
}

impl StyleForTrace for TraceEntry {
    fn get_style(&self, index: u64) -> Style {
        self.get_style_with_bg(index, None)
    }
    fn get_style_with_bg(&self, index: u64, bg: Option<Color>) -> Style {
        let mut style = Style::new();
        if self.index <= index && self.index_stop >= index {
            style.bg = bg;
        }
        if self.traces.len() == 0 {
            if self.index <= index && self.index_stop >= index {
                style.fg = Some(HEX_COLORS[0]);
            }
        } else {
            for (i, t) in self.traces.iter().enumerate() {
                if t.index <= index && t.index_stop >= index {
                    style.fg = Some(HEX_COLORS[i % 4]);
                }
            }
        }
        style
    }
}

impl App<'_> {
    fn render_value(&mut self, area: Rect, buf: &mut Buffer) {
        // FIX: we need to handle the case where there are no traces better
        let ct = match self.trace_get_current_highlighted() {
            Some(t) => t,
            None => return,
        };
        let op = format!("{:?} {:?}", ct.info, ct.value);
        let chunks: Vec<Line> = op
            .chars()
            .collect::<Vec<char>>()
            .chunks(area.width as usize)
            .map(|chunk| {
                let c: String = chunk.iter().collect();
                Line::from(Span::default().content(c))
            })
            .collect();

        let p = Paragraph::new(chunks);
        p.render(area, buf);
    }

    fn render_hex(&mut self, area: Rect, buf: &mut Buffer) {
        if area.width > 148 {
            self.hex.width = 32;
        } else {
            self.hex.width = 16;
        }
        // FIX: we need to handle the case where there are no traces better
        let t = match self.trace_get_current_highlighted() {
            Some(t) => t,
            None => return,
        };
        let trace_start = t.index;
        let rest = t.index % self.hex.width;
        let display_start = trace_start - rest;
        let default_style = Style::new();
        let mut lines = vec![];
        for i in 0..area.height - 1 {
            let d = display_start as u64 + i as u64 * self.hex.width;
            let mut h: Vec<Span> = vec![];
            h.push(format!("0x{:012} ", d).into());
            for r_i in 0..self.hex.width {
                let di = (d + r_i) as usize;
                if di >= self.data.len() {
                    h.push(Span::styled(format!("   "), default_style));
                } else {
                    let style = t.get_style(di as u64);
                    h.push(Span::styled(format!("{:02X} ", self.data[di]), style));
                }
            }
            h.push(format!(" | ").into());
            for r_i in 0..self.hex.width {
                let di = (d + r_i) as usize;

                if di >= self.data.len() {
                    h.push(Span::styled(format!(" "), default_style));
                } else {
                    let c = self.data[di] as char;
                    let ascii_representation = if c.is_ascii_graphic() {
                        c
                    } else if c.is_ascii_whitespace() {
                        ' '
                    } else {
                        '.'
                    };
                    let style = t.get_style_with_bg(di as u64, Some(TODO_HEADER_BG));
                    h.push(Span::styled(format!("{}", ascii_representation), style));
                }
            }
            h.push(format!(" | ").into());
            let l = Line::from(h);
            lines.push(l);
        }
        let p = Paragraph::new(lines);
        p.render(area, buf);
    }

    fn render_tracelist_v2(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let time_start = Instant::now();
        let index = self.index_current();
        let (trace_index, index_offset) = self.trace_get_with_offset((area.height - 1) as usize);
        self.trace_set_last_selected(index_offset);
        let read = if self.trace.trace.traces.len() > 0 {
            let i = self.trace.trace.traces.len() - 1;
            self.trace.trace.traces[i].index_stop
        } else {
            0
        };

        let data_len = self.data.len();

        let time_start_items = Instant::now();
        let mut items: Vec<ListItem>;
        let mut inner_block: Block;
        let mut inner_area = Rect::default();
        let mut iter_stop = 0;

        {
            let debug = &mut self.debug;
            debug.tracelist_add("time_start_items", time_start_items.elapsed());
            debug.tracelist_add("trace_index", trace_index);
            debug.tracelist_add("iter_stop", iter_stop);
        }

        let inner_info_block = Block::default()
            .borders(Borders::NONE)
            .bg(NORMAL_ROW_COLOR)
            .padding(Padding::horizontal(1));

        let last_selected = self.get_last_selected() as usize;
        let info_paragraph = match self.trace_get_current_selected() {
            Some(current_trace) => {
                let mut tl = vec![];
                iter_stop =
                    if trace_index + (area.height - 1) as usize >= current_trace.traces.len() {
                        if current_trace.traces.len() > 0 {
                            current_trace.traces.len()
                        } else {
                            0
                        }
                    } else {
                        trace_index + (area.height) as usize
                    };

                let mut style = Style::new();
                let line = Line::styled(
                    format!(
                        "{}/{} {}",
                        last_selected + trace_index + 1,
                        current_trace.traces.len(),
                        data_len
                    ),
                    style,
                );
                tl.push(line);

                #[derive(Debug, Clone)]
                struct ColumnWidths {
                    field_type: usize,
                    field_name: usize,
                    index: usize,
                    index_stop: usize,
                };

                impl ColumnWidths {
                    fn width(&self) -> usize {
                        self.field_type + self.field_name + self.index + self.index_stop + 3 * 3
                    }
                    fn grow(&mut self) {
                        self.field_type += 1;
                        self.field_name += 1;
                        self.index += 1;
                        self.index_stop += 1;
                    }

                    fn grow_by(&mut self, grow: usize) {
                        self.field_type += grow;
                        self.field_name += grow;
                        self.index += grow;
                        self.index_stop += grow;
                    }
                };

                let mut column_widths = ColumnWidths {
                    field_type: "type".len(),
                    field_name: "name".len(),
                    index: "start".len(),
                    index_stop: "stop".len(),
                };

                for (index, item) in current_trace.traces[trace_index..iter_stop]
                    .iter()
                    .enumerate()
                {
                    column_widths
                        .field_type
                        .assign_if_greater(item.field_type.len());
                    column_widths
                        .field_name
                        .assign_if_greater(item.field_name.len());
                    column_widths
                        .index
                        .assign_if_greater(format!("{}", item.index).len());
                    column_widths
                        .index_stop
                        .assign_if_greater(format!("{}", item.index_stop).len());
                }

                let mut remaining_size = area.width as usize - column_widths.width();
                let column_widths_before = column_widths.clone();
                column_widths.grow_by((remaining_size / 4) - 1);

                let cft = column_widths.field_type;
                let cfn = column_widths.field_name;
                let ci = column_widths.index;
                let cis = column_widths.index_stop;

                let mut style = Style::new();
                let line = Line::styled(
                    format!(
                        "{:^cft$} | {:^cfn$} | {:^ci$} | {:^cis$}",
                        "type", "name", "start", "stop"
                    ),
                    style,
                );
                tl.push(line);

                for (index, item) in current_trace.traces[trace_index..iter_stop]
                    .iter()
                    .enumerate()
                {
                    let mut style = Style::new();
                    let bg_color = match index % 2 {
                        0 => NORMAL_ROW_COLOR,
                        _ => ALT_ROW_COLOR,
                    };
                    style.bg = Some(bg_color);
                    style.fg = Some(TEXT_COLOR);

                    if last_selected == index {
                        style.bg = Some(SELECTED_STYLE_FG);
                        style.fg = Some(SELECTED_TEXT_COLOR);
                    }

                    let cft = column_widths.field_type;
                    let cfn = column_widths.field_name;
                    let ci = column_widths.index;
                    let cis = column_widths.index_stop;

                    let line = Line::styled(
                        format!(
                            "{:<cft$} | {:<cfn$} | {:>ci$} | {:>cis$}",
                            item.field_type, item.field_name, item.index, item.index_stop
                        ),
                        style,
                    );
                    tl.push(line);
                }

                {
                    let debug = &mut self.debug;
                    debug.tracelist_add(
                        "columns",
                        format!(
                            "{} {} {}",
                            area.width,
                            remaining_size,
                            column_widths.width()
                        ),
                    );
                    debug.tracelist_add("column_widths", format!("{:?}", column_widths));
                    debug.tracelist_add(
                        "column_widths_before",
                        format!("{:?}", column_widths_before),
                    );
                }
                Paragraph::new(tl)
                    .block(inner_info_block)
                    .fg(TEXT_COLOR)
                    .wrap(Wrap { trim: false })
            }
            None => Paragraph::new("implement me")
                .block(inner_info_block)
                .fg(TEXT_COLOR)
                .wrap(Wrap { trim: false }),
        };

        // We can now render the subtraces info
        info_paragraph.render(area, buf);

        {
            let debug = &mut self.debug;
            let passed = time_start.elapsed();
            debug.tracelist_add("render v2", passed);
        }
    }

    fn render_tracelist(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let time_start = Instant::now();
        let index = self.index_current();
        let (trace_index, index_offset) = self.trace_get_with_offset((area.height - 1) as usize);
        self.trace_set_last_selected(index_offset);
        let read = if self.trace.trace.traces.len() > 0 {
            let i = self.trace.trace.traces.len() - 1;
            self.trace.trace.traces[i].index_stop
        } else {
            0
        };

        let time_start_items = Instant::now();
        let mut items: Vec<ListItem>;
        let mut inner_block: Block;
        let mut inner_area = Rect::default();
        let mut iter_stop = 0;
        {
            let last_selected = self.get_last_selected() as usize;
            match self.trace_get_current_selected() {
                Some(current_trace) => {
                    let s = format!(
                        "traces: {}/{} pos: {}/{}",
                        index + 1,
                        current_trace.traces.len(),
                        current_trace.index,
                        read
                    );
                    let outer_block = Block::default()
                        .borders(Borders::NONE)
                        .fg(TEXT_COLOR)
                        .bg(TODO_HEADER_BG)
                        .title(s)
                        .title_alignment(Alignment::Left);
                    inner_block = Block::default()
                        .borders(Borders::NONE)
                        .fg(TEXT_COLOR)
                        .bg(NORMAL_ROW_COLOR);

                    // We get the inner area from outer_block. We'll use this area later to render the table.
                    let outer_area = area;
                    inner_area = outer_block.inner(outer_area);

                    // We can render the header in outer_area.
                    outer_block.render(outer_area, buf);

                    // Iterate through all elements in the `items` and stylize them.
                    iter_stop = if trace_index + area.height as usize >= current_trace.traces.len()
                    {
                        if current_trace.traces.len() > 0 {
                            current_trace.traces.len()
                        } else {
                            0
                        }
                    } else {
                        trace_index + area.height as usize
                    };

                    items = current_trace.traces[trace_index..iter_stop]
                        .iter()
                        .enumerate()
                        .map(|(i, todo_item)| todo_item.clone().to_list_item(i, last_selected))
                        .collect();
                }
                // FIX: we need to handle the case where there are no traces better
                None => return,
            };
        }

        {
            let debug = &mut self.debug;
            debug.tracelist_add("time_start_items".to_string(), time_start_items.elapsed());
            debug.tracelist_add("trace_index".to_string(), trace_index);
            debug.tracelist_add("iter_stop".to_string(), iter_stop);
        }

        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        // (look careful we are using StatefulWidget's render.)
        // ratatui::widgets::StatefulWidget::render as stateful_render
        //StatefulWidget::render(items, inner_area, buf, &mut self.state);
        Widget::render(items, inner_area, buf);

        {
            let debug = &mut self.debug;
            let passed = time_start.elapsed();
            debug.tracelist_add("render".to_string(), passed);
        }
    }

    fn render_subtraces(&mut self, area: Rect, buf: &mut Buffer) {
        let start = Instant::now();
        let t = match self.trace_get_current_highlighted() {
            Some(t) => t,
            // FIX: we need to handle the case where there are no traces better
            None => return,
        };
        let outer_info_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG)
            .title(format!(
                "field name: ({}) - sub({})",
                t.field_name,
                t.traces.len(),
            ))
            .title_alignment(Alignment::Center);
        let inner_info_block = Block::default()
            .borders(Borders::NONE)
            .bg(NORMAL_ROW_COLOR)
            .padding(Padding::horizontal(1));

        // This is a similar process to what we did for list. outer_info_area will be used for
        // header inner_info_area will be used for the list info.
        let outer_info_area = area;
        let inner_info_area = outer_info_block.inner(outer_info_area);

        // We can render the header. Inner info will be rendered later
        outer_info_block.render(outer_info_area, buf);

        // FIX: we need to handle the case where there are no traces better
        let t = match self.trace_get_current_selected() {
            Some(t) => t,
            None => return,
        };

        let mut max_len = 0;
        for trace in t.traces.iter() {
            if trace.field_name.len() > max_len {
                max_len = trace.field_name.len();
            }
        }

        let info_paragraph = match self.trace_get_current_highlighted() {
            Some(current_selected_trace) => {
                let mut tl = vec![];
                if current_selected_trace.traces.len() > 0 {
                    for (i, trace) in current_selected_trace.traces.iter().enumerate() {
                        let mut style = Style::new();
                        style.fg = Some(HEX_COLORS[i % 4]);
                        tl.push(Line::styled(
                            format!(
                                "{:width$}: {} \n",
                                trace.field_name,
                                trace.field_type,
                                width = max_len,
                            ),
                            style,
                        ));
                    }
                } else {
                    let style = Style::new();
                    tl.push(Line::styled(
                        format!("field_name: {}", current_selected_trace.field_name),
                        style,
                    ));
                    tl.push(Line::styled(
                        format!("value: {:?}", current_selected_trace.value),
                        style,
                    ));
                }
                Paragraph::new(tl)
                    .block(inner_info_block)
                    .fg(TEXT_COLOR)
                    .wrap(Wrap { trim: false })
            }
            None => Paragraph::new("implement me")
                .block(inner_info_block)
                .fg(TEXT_COLOR)
                .wrap(Wrap { trim: false }),
        };

        // We can now render the subtraces info
        info_paragraph.render(inner_info_area, buf);
        let stop = start.elapsed();
        self.debug.subtraces_add("render", stop);
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Traceview")
            .bold()
            .centered()
            .render(area, buf);
    }
    fn render_debug(&mut self, area: Rect, buf: &mut Buffer) {
        let mut il = format!("initialization:\n");
        for (k, v) in &self.debug.initialization {
            il = il + &format!("   {}: {:?}\n", k, v);
        }

        let mut tl = format!("tracelist:\n");
        for (k, v) in &self.debug.tracelist {
            tl = tl + &format!("   {}: {:?}\n", k, v);
        }
        let mut st = format!("subtraces:\n");
        for (k, v) in &self.debug.subtraces {
            st = st + &format!("   {}: {:?}\n", k, v);
        }

        // FIX: we need to handle the case where there are no traces better
        let ct = match self.trace_get_current_selected() {
            Some(t) => t,
            None => return,
        };
        let e = format!("curent trace: {}/{}", ct.index, ct.index_stop);
        let d = format!("{}{}{}{}", tl, st, il, e);
        Paragraph::new(d).bold().left_aligned().render(area, buf);
    }
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new("\nF1 to show help")
        .centered()
        .render(area, buf);
}

#[allow(dead_code)]
#[derive(Debug)]
enum DebugValue {
    String(String),
    Duration(std::time::Duration),
}

impl From<String> for DebugValue {
    fn from(value: String) -> Self {
        DebugValue::String(value)
    }
}

impl From<std::time::Duration> for DebugValue {
    fn from(value: std::time::Duration) -> Self {
        DebugValue::Duration(value)
    }
}
