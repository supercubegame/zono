// zono v0.2.1 - a tiny cross-platform todo list with a GUI
// Built with iced 0.12 (Sandbox API): native Win32, Cocoa and X11/Wayland windows.

use iced::widget::{Button, Column, Container, Row, Scrollable, Text, TextInput};
use iced::{Alignment, Element, Length, Sandbox, Settings};

use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
struct Task {
    id: u32,
    done: bool,
    title: String,
}

fn data_dir() -> PathBuf {
    let base = env::var("ZONO_HOME")
        .ok()
        .or_else(|| env::var("HOME").ok())
        .or_else(|| env::var("USERPROFILE").ok())
        .unwrap_or_else(|| String::from("."));
    let mut path = PathBuf::from(base);
    path.push(".zono");
    path
}

fn data_file() -> PathBuf {
    let mut path = data_dir();
    path.push("tasks.tsv");
    path
}

// Same tab-separated format the CLI version used, so data carries over.
fn load() -> Vec<Task> {
    let mut tasks = Vec::new();
    if let Ok(text) = fs::read_to_string(data_file()) {
        for line in text.lines() {
            let mut parts = line.splitn(3, '\t');
            let raw_id = parts.next().unwrap_or("").trim().to_string();
            let raw_done = parts.next().unwrap_or("0").trim().to_string();
            let title = parts.next().unwrap_or("").to_string();
            if title.is_empty() {
                continue;
            }
            if let Ok(id) = raw_id.parse::<u32>() {
                tasks.push(Task {
                    id,
                    done: raw_done == "1",
                    title,
                });
            }
        }
    }
    tasks
}

fn save(tasks: &[Task]) -> std::io::Result<()> {
    fs::create_dir_all(data_dir())?;
    let mut body = String::new();
    for task in tasks {
        let flag = if task.done { "1" } else { "0" };
        body.push_str(&format!("{}\t{}\t{}\n", task.id, flag, task.title));
    }
    fs::write(data_file(), body)
}

fn next_id(tasks: &[Task]) -> u32 {
    tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

#[derive(Default)]
struct Zono {
    tasks: Vec<Task>,
    input: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    AddTask,
    ToggleTask(u32),
    DeleteTask(u32),
    ClearDone,
}

impl Sandbox for Zono {
    type Message = Message;

    fn new() -> Self {
        Self {
            tasks: load(),
            input: String::new(),
        }
    }

    fn title(&self) -> String {
        String::from("zono - todo list")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input = value;
            }
            Message::AddTask => {
                let title = self.input.trim().to_string();
                if !title.is_empty() {
                    let id = next_id(&self.tasks);
                    self.tasks.push(Task {
                        id,
                        done: false,
                        title,
                    });
                    self.input.clear();
                    let _ = save(&self.tasks);
                }
            }
            Message::ToggleTask(id) => {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    task.done = !task.done;
                    let _ = save(&self.tasks);
                }
            }
            Message::DeleteTask(id) => {
                self.tasks.retain(|t| t.id != id);
                let _ = save(&self.tasks);
            }
            Message::ClearDone => {
                self.tasks.retain(|t| !t.done);
                let _ = save(&self.tasks);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = Column::new()
            .push(Text::new("zono").size(34))
            .push(Text::new("v0.2.1 - todo list with GUI").size(14))
            .spacing(2);

        let input = TextInput::new("Add a new task...", &self.input)
            .on_input(Message::InputChanged)
            .on_submit(Message::AddTask)
            .padding(10)
            .size(16);

        let input_row = Row::new()
            .push(input)
            .push(
                Button::new(Text::new("Add").size(16))
                    .on_press(Message::AddTask)
                    .padding(10),
            )
            .spacing(8)
            .align_items(Alignment::Center);

        let mut list = Column::new().spacing(6);

        if self.tasks.is_empty() {
            list = list.push(Text::new("No tasks yet. Add one above.").size(15));
        } else {
            for task in &self.tasks {
                let box_label = if task.done { "[x]" } else { "[ ]" };

                let row = Row::new()
                    .push(
                        Button::new(Text::new(box_label).size(15))
                            .on_press(Message::ToggleTask(task.id))
                            .padding(6),
                    )
                    .push(
                        Container::new(Text::new(&task.title).size(16))
                            .width(Length::Fill)
                            .padding(4),
                    )
                    .push(
                        Button::new(Text::new("Del").size(14))
                            .on_press(Message::DeleteTask(task.id))
                            .padding(6),
                    )
                    .spacing(8)
                    .align_items(Alignment::Center);

                list = list.push(row);
            }
        }

        let done = self.tasks.iter().filter(|t| t.done).count();
        let total = self.tasks.len();

        let footer = Row::new()
            .push(
                Container::new(Text::new(format!("{} of {} done", done, total)).size(14))
                    .width(Length::Fill),
            )
            .push(
                Button::new(Text::new("Clear Finished").size(14))
                    .on_press(Message::ClearDone)
                    .padding(8),
            )
            .spacing(10)
            .align_items(Alignment::Center);

        let content = Column::new()
            .push(header)
            .push(input_row)
            .push(Scrollable::new(list).height(Length::Fill))
            .push(footer)
            .spacing(16)
            .padding(22)
            .max_width(520);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

fn main() -> iced::Result {
    Zono::run(Settings::default())
}
