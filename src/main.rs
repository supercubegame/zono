// zono v0.2.0 - a tiny cross-platform todo list with GUI
// Built with iced: native Windows (WinForms), macOS (Cocoa), and Linux (GTK) UIs

use iced::{
    alignment, button, container, text, text_input, Alignment, Button, Column, Command,
    Container, Element, Length, Row, Sandbox, Settings, Text, TextInput, Application,
};
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
    input_state: text_input::State,
    add_btn_state: button::State,
    delete_btn_states: Vec<button::State>,
    toggle_btn_states: Vec<button::State>,
    clear_btn_state: button::State,
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
        let tasks = load();
        let delete_btn_states = vec![button::State::new(); tasks.len()];
        let toggle_btn_states = vec![button::State::new(); tasks.len()];
        Self {
            tasks,
            input: String::new(),
            input_state: text_input::State::new(),
            add_btn_state: button::State::new(),
            delete_btn_states,
            toggle_btn_states,
            clear_btn_state: button::State::new(),
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
                if !self.input.trim().is_empty() {
                    let id = next_id(&self.tasks);
                    self.tasks.push(Task {
                        id,
                        done: false,
                        title: self.input.trim().to_string(),
                    });
                    self.input.clear();
                    self.delete_btn_states.push(button::State::new());
                    self.toggle_btn_states.push(button::State::new());
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
                self.delete_btn_states.pop();
                self.toggle_btn_states.pop();
                let _ = save(&self.tasks);
            }
            Message::ClearDone => {
                let before = self.tasks.len();
                self.tasks.retain(|t| !t.done);
                let diff = before - self.tasks.len();
                self.delete_btn_states.truncate(self.tasks.len());
                self.toggle_btn_states.truncate(self.tasks.len());
                let _ = save(&self.tasks);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("zono").size(32);
        let subtitle = Text::new("v0.2.0 - todo list with GUI").size(14);

        let input = TextInput::new(
            &mut self.input_state,
            "Add a new task...",
            &self.input,
            Message::InputChanged,
        )
        .padding(10);

        let add_btn = Button::new(&mut self.add_btn_state, Text::new("Add"))
            .on_press(Message::AddTask)
            .padding(10);

        let input_row = Row::new().push(input).push(add_btn).spacing(10);

        let mut task_list = Column::new().spacing(8).padding(10);

        if self.tasks.is_empty() {
            task_list = task_list.push(Text::new("No tasks yet. Add one above!"));
        } else {
            for (idx, task) in self.tasks.iter().enumerate() {
                let checkbox = Button::new(
                    &mut self.toggle_btn_states[idx],
                    Text::new(if task.done { "✓" } else { "○" }),
                )
                .on_press(Message::ToggleTask(task.id))
                .padding(5);

                let task_text = if task.done {
                    Text::new(&task.title).size(16)
                } else {
                    Text::new(&task.title).size(16)
                };

                let delete_btn = Button::new(
                    &mut self.delete_btn_states[idx],
                    Text::new("🗑"),
                )
                .on_press(Message::DeleteTask(task.id))
                .padding(5);

                let task_row = Row::new()
                    .push(checkbox)
                    .push(task_text)
                    .push(delete_btn)
                    .spacing(10)
                    .align_items(Alignment::Center);

                task_list = task_list.push(task_row);
            }
        }

        let done_count = self.tasks.iter().filter(|t| t.done).count();
        let total_count = self.tasks.len();
        let progress = Text::new(format!("{} of {} done", done_count, total_count)).size(14);

        let clear_btn = Button::new(&mut self.clear_btn_state, Text::new("Clear Finished"))
            .on_press(Message::ClearDone)
            .padding(10);

        let footer = Row::new()
            .push(progress)
            .push(clear_btn)
            .spacing(20)
            .padding(10);

        let content = Column::new()
            .push(title)
            .push(subtitle)
            .push(input_row)
            .push(task_list)
            .push(footer)
            .spacing(15)
            .padding(20);

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
