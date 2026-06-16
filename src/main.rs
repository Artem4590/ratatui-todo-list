use directories::ProjectDirs;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, HighlightSpacing, List, ListItem, ListState},
};
use std::path::{Path, PathBuf};

use std::{fs, io, vec};

use serde::{Deserialize, Serialize};

const SELECTED_STYLE: Style = Style::new();

/// 1️⃣ СОСТОЯНИЕ ПРИЛОЖЕНИЯ
///
/// Здесь хранятся ВСЕ данные.
/// Никакой логики отрисовки, только состояние.

#[derive(Serialize, Deserialize, Debug)]
struct PersistedState {
    todo_list: TodoList,
}

pub struct App {
    exit: bool,
    todo_list: TodoList,
    mode: Mode,
    state: ListState,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoList {
    items: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    text: String,
    status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    Todo,
    Completed,
}

enum Mode {
    Normal,
    Adding,
    Editing,
}

impl Mode {
    fn as_str(&self) -> Span<'_> {
        match self {
            Mode::Normal => Span::from("NORMAL"),
            Mode::Adding => Span::from("ADDING"),
            Mode::Editing => Span::from("EDITING"),
        }
    }
}

impl Status {
    fn as_str(&self) -> Span<'_> {
        match self {
            Status::Todo => Span::from("☐"),
            Self::Completed => Span::from("✓").green(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            todo_list: TodoList {
                items: vec![
                    TodoItem {
                        text: String::from("Hello1"),
                        status: Status::Todo,
                    },
                    TodoItem {
                        text: String::from("Hello2"),
                        status: Status::Completed,
                    },
                    TodoItem {
                        text: String::from("Hello3"),
                        status: Status::Todo,
                    },
                    TodoItem {
                        text: String::from("Hello from v0.2.0"),
                        status: Status::Todo,
                    },
                ],
            },
            state: ListState::default(),
            mode: Mode::Normal,
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut app = App::default();
    // ratatui::run:
    // - включает raw mode
    // - переключает экран
    // - гарантирует cleanup
    ratatui::run(|terminal| app.run(terminal))
}

impl App {
    /// ГЛАВНЫЙ ЦИКЛ ПРОГРАММЫ
    ///
    /// Пока exit == false:
    ///   - рисуем
    ///   - читаем ввод
    ///   - обновляем состояние
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.state.select(Some(0));
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Метод реализует отрисовку
    ///
    /// ВАЖНО:
    /// - НЕ менять состояние
    /// - ТОЛЬКО читать self
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area(); // получаем область рендера

        let title = Line::from(" Todo-list ".bold());

        let items: Vec<ListItem> = self
            .todo_list
            .items
            .iter()
            .map(|todo_item| {
                ListItem::from(Line::from_iter([
                    todo_item.status.as_str(),
                    Span::from(" "),
                    Span::from(&todo_item.text),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(Span::from("=> ").blue())
            .highlight_spacing(HighlightSpacing::Always);

        let instructions = Line::from(vec![
            " quit ".into(),
            "<q> ".blue().bold(),
            " edit ".into(),
            "<e> ".blue().bold(),
            " add ".into(),
            "<ctrl+a> ".blue().bold(),
            " delete ".into(),
            "<ctrl+d> ".blue().bold(),
            " completed ".into(),
            "<space> ".blue().bold(),
        ]);

        let instructions_in_mode = Line::from(vec![
            " quit ".into(),
            "<q> ".blue().bold(),
            " apply ".into(),
            "<esc> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(Line::from(format!(" mode: {} ", self.mode.as_str())).left_aligned())
            .title_bottom(match self.mode {
                Mode::Normal => instructions.centered(),
                _ => instructions_in_mode.centered(),
            });

        let inner_block = block.inner(area);
        frame.render_widget(block, area); // вызываем рендер
        frame.render_stateful_widget(list, inner_block, &mut self.state);
    }

    /// Метод реализует обработку событий ввода
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_event(key),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.mode {
            Mode::Normal => self.handle_key_normal(key),
            Mode::Editing => self.handle_key_input(key),
            Mode::Adding => self.handle_key_input(key),
        }
    }

    fn handle_key_normal(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.exit = true;
            }
            KeyCode::Char('e') => {
                self.start_edit();
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                let selected_item = self.state.selected();
                if let Some(item_index) = selected_item {
                    self.todo_list.items.remove(item_index);
                }
            }
            KeyCode::Char('a') if key.modifiers == KeyModifiers::CONTROL => {
                self.start_add();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Char(' ') => {
                let selected_item = self.state.selected();
                if let Some(item_index) = selected_item {
                    match self.todo_list.items[item_index].status {
                        Status::Todo => self.todo_list.items[item_index].status = Status::Completed,
                        Status::Completed => self.todo_list.items[item_index].status = Status::Todo,
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> io::Result<()> {
        let selected_item = self.state.selected();
        match key.code {
            KeyCode::Char(c) if !c.is_control() => {
                if let Some(item_index) = selected_item {
                    self.todo_list.items[item_index].text.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = selected_item {
                    self.todo_list.items[i].text.pop();
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn start_edit(&mut self) {
        if self.todo_list.items.is_empty() {
            return;
        }
        self.mode = Mode::Editing;
    }

    fn start_add(&mut self) {
        self.mode = Mode::Adding;
        let selected_item = self.state.selected();
        if let Some(item_index) = selected_item {
            let new_item_index = item_index + 1;
            self.todo_list.items.insert(
                new_item_index,
                TodoItem {
                    text: String::new(),
                    status: Status::Todo,
                },
            );
            self.state.select(Some(new_item_index));
        }
    }
}
