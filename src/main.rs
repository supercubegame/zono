// zono - a tiny cross-platform todo list.
// Zero dependencies: std only, so it builds on Linux, macOS and Windows
// with nothing but a stock Rust toolchain.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

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

fn save(tasks: &[Task]) -> io::Result<()> {
    fs::create_dir_all(data_dir())?;
    let mut body = String::new();
    for task in tasks {
        let flag = if task.done { "1" } else { "0" };
        body.push_str(&format!("{}\t{}\t{}\n", task.id, flag, task.title));
    }
    fs::write(data_file(), body)
}

fn banner() {
    println!("+----------------------------------------------+");
    println!("|  zono  ::  a tiny cross-platform todo list    |");
    println!("|  v0.1.0 :: rust, zero dependencies            |");
    println!("+----------------------------------------------+");
}

fn help() {
    println!();
    println!("  a <title>   add a task");
    println!("  d <id>      toggle done / not done");
    println!("  r <id>      remove a task");
    println!("  c           clear every finished task");
    println!("  h           show this help");
    println!("  q           save and quit");
}

fn show(tasks: &[Task]) {
    println!();
    println!("  YOUR TASKS");
    println!("  ----------------------------------------------");
    if tasks.is_empty() {
        println!("  nothing here yet, add your first task with: a");
    }
    for task in tasks {
        let mark = if task.done { "[x]" } else { "[ ]" };
        println!("  {:>3}. {} {}", task.id, mark, task.title);
    }
    let done = tasks.iter().filter(|t| t.done).count();
    println!("  ----------------------------------------------");
    println!("  {} of {} done", done, tasks.len());
}

fn next_id(tasks: &[Task]) -> u32 {
    tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

fn main() {
    let mut tasks = load();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    banner();
    help();

    loop {
        show(&tasks);
        print!("\nzono> ");
        let _ = io::stdout().flush();

        let raw = match lines.next() {
            Some(Ok(line)) => line,
            _ => {
                println!("\nsaved. bye!");
                break;
            }
        };
        let input = raw.trim().to_string();
        println!("{}", input);
        if input.is_empty() {
            continue;
        }

        let (cmd, rest) = match input.find(' ') {
            Some(i) => (&input[..i], input[i + 1..].trim()),
            None => (input.as_str(), ""),
        };

        match cmd {
            "a" | "add" => {
                if rest.is_empty() {
                    println!("usage: a <task title>");
                } else {
                    let id = next_id(&tasks);
                    tasks.push(Task {
                        id,
                        done: false,
                        title: rest.to_string(),
                    });
                    println!("added #{}", id);
                }
            }
            "d" | "done" => match rest.parse::<u32>() {
                Ok(id) => {
                    let mut hit = false;
                    for task in tasks.iter_mut() {
                        if task.id == id {
                            task.done = !task.done;
                            hit = true;
                        }
                    }
                    if hit {
                        println!("toggled #{}", id);
                    } else {
                        println!("no task #{}", id);
                    }
                }
                Err(_) => println!("usage: d <id>"),
            },
            "r" | "rm" => match rest.parse::<u32>() {
                Ok(id) => {
                    let before = tasks.len();
                    tasks.retain(|t| t.id != id);
                    if tasks.len() == before {
                        println!("no task #{}", id);
                    } else {
                        println!("removed #{}", id);
                    }
                }
                Err(_) => println!("usage: r <id>"),
            },
            "c" | "clear" => {
                let before = tasks.len();
                tasks.retain(|t| !t.done);
                println!("cleared {} finished task(s)", before - tasks.len());
            }
            "h" | "help" => help(),
            "q" | "quit" => {
                let _ = save(&tasks);
                println!("saved to {}. bye!", data_file().display());
                break;
            }
            other => println!("unknown command: {} (type h for help)", other),
        }

        let _ = save(&tasks);
    }
}
