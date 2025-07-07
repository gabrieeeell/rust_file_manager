use cli_log;
use std::{
    default, env, fs,
    io::{self, BufRead},
    path::{self, PathBuf},
};
mod file_manager;
mod for_debug;
use color_eyre::Result;
use file_manager::{FileItem, FileList, list_dir};
use ratatui::{
    DefaultTerminal, Terminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{BLUE, GREEN, SLATE},
    },
    symbols,
    text::Line,
    widgets::{
        self, Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use size::Size;
use tracing::debug;

const LIST_BG_COLOR: Color = SLATE.c950;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

fn main() -> Result<()> {
    unsafe {
        for_debug::initialize_logging()?;
    }
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    Ok(())
}

struct App {
    should_exit: bool,
    file_list: FileList,
    curr_path: String,
    initial_full_path: String,
    curr_file: Option<String>, //  A string because if i save a File i cant get the name back xddd
                               //  I could use the FileItem but for now i dont really need the metadata (i think)
}

impl Default for App {
    //this refers to the creation of a default instance of App probably
    fn default() -> Self {
        Self {
            should_exit: false,
            file_list: FileList::from_path(&String::from("./")), // because it is the default, "./" should
            curr_path: String::from("./"),                       // have sense
            initial_full_path: env::current_dir().unwrap().to_str().unwrap().to_string(),
            curr_file: None,
        }
    }
}

impl App {
    // maybe here self is a mut move because the App isnt used anywhere else?
    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            // Always after the render is checked if any key is pressed
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('l') | KeyCode::Enter => self.enter_file(),
            KeyCode::Char('h') | KeyCode::Backspace => self.go_back(),
            // Here would be something for going back
            _ => {}
        }
    }

    fn select_next(&mut self) {
        self.file_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.file_list.state.select_previous();
    }
    // if the last directory visit is different from "." or ".." a "../" is added else, the path is
    // erased to the most recent "/"
    fn go_back(&mut self) {
        if self.curr_file.is_none() {
            if self.curr_path != "./" {
                let path_without_last_parenthesis = &self.curr_path[..self.curr_path.len() - 1];
                let last_visited_file = &path_without_last_parenthesis
                    [path_without_last_parenthesis.rfind("/").unwrap() + 1..];

                //maybe i should check if this is the last directory or sm
                self.curr_path = if last_visited_file == ".." {
                    self.curr_path.clone() + "../"
                } else {
                    (path_without_last_parenthesis
                        [..path_without_last_parenthesis.rfind("/").unwrap()])
                        .to_owned()
                        + "/"
                };
            } else {
                self.curr_path = self.curr_path.clone() + "../";
            }
            self.file_list.files = list_dir(&self.curr_path).unwrap();
            self.file_list.state.select(None);
        } else {
            self.curr_file = None;
        }
    }
    fn enter_file(&mut self) {
        // if let some necessary because could be no item selected
        if self.curr_file.is_none() {
            if let Some(i) = self.file_list.state.selected() {
                if self.file_list.files[i].is_dir() == true {
                    self.curr_path = self.curr_path.clone() + self.file_list.files[i].name() + "/";
                    self.file_list.files = list_dir(&self.curr_path).unwrap();
                    self.file_list.state.select(None);
                } else {
                    self.curr_file = Some(self.file_list.files[i].name().to_string());
                }
            };
        }
    }
}

impl Widget for &mut App {
    //render() is in charge of defining what happens visually when something is selected
    //The buffer is like implicitly passed(?
    fn render(self, area: Rect, buf: &mut Buffer) {
        //test a bit with layout and constraints
        let layout =
            Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area);

        self.render_list(layout[0], buf);
        self.render_item_info(layout[1], buf);
    }
}

//Another impl for renders methods, for organization probably

fn convert_to_absolute_path(mut initial_path: String, mut relative_path: String) -> String {
    if !relative_path.contains("..") {
        initial_path + &relative_path[1..]
    } else {
        relative_path = (relative_path[2..]).to_string();
        relative_path = relative_path.chars().rev().collect();
        while relative_path.contains("..") {
            relative_path = (relative_path[..relative_path.rfind("/..").unwrap()]).to_string();
            initial_path = (initial_path[..initial_path.rfind("/").unwrap()]).to_string();
        }
        relative_path = relative_path.chars().rev().collect();
        initial_path + "/" + &relative_path
    }
}

impl App {
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let mut list_title =
            convert_to_absolute_path(self.initial_full_path.clone(), self.curr_path.clone());
        if self.curr_file.is_some() {
            list_title += self.curr_file.as_ref().unwrap()
        };

        let block = Block::new().title(Line::raw(list_title)).bg(LIST_BG_COLOR);
        //check if there is no border by not putting nothing of borders

        let items: Vec<ListItem> = if self.curr_file.is_none() {
            //self.file_list.to_formated_list_item() //couldnt make it work

            self.file_list
                .files
                .iter()
                .map(|file| {
                    ListItem::from(format!(
                        "{:}                              {:}",
                        file.name(),
                        file.size()
                    ))
                })
                .collect()
        } else {
            io::BufReader::new(
                fs::File::open(self.curr_path.clone() + self.curr_file.as_ref().unwrap()).unwrap(),
            )
            .lines()
            .map_while(Result::ok)
            .map(ListItem::from) // same as |line|
            .collect() // ListItem::from(line)
        };

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::WhenSelected);
        StatefulWidget::render(list, area, buf, &mut self.file_list.state);
    }
    fn render_item_info(&mut self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(String::from(&self.curr_path));
        Widget::render(paragraph, area, buf);
    }
}
