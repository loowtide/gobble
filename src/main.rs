use crossterm::style::{Color, Stylize, style};
//use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use dotenv;
use gemini_rust::Gemini;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{ColorMode, Config, Editor, Helper};
use std::env;
use std::{
    error::Error,
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};
mod dashboard;
use tokio;

struct ShellHelper {
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    validator: MatchingBracketValidator,
}

impl Helper for ShellHelper {}
impl Completer for ShellHelper {
    type Candidate = String;
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        if line.trim().is_empty() {
            return line.into();
        }

        let colored_line = if line.starts_with("cd ") {
            let (cmd, path) = line.split_at(3);
            format!(
                "{}{}",
                style(cmd).with(Color::Blue),
                style(path).with(Color::Green)
            )
        } else if line.starts_with("exit") {
            style(line).with(Color::Red).to_string()
        } else if line.contains(" | ") {
            let parts: Vec<&str> = line.split(" | ").collect();
            parts
                .iter()
                .map(|part| style(part).with(Color::Yellow).to_string())
                .collect::<Vec<String>>()
                .join(&style(" | ").with(Color::Magenta).to_string())
        } else {
            style(line).with(Color::Yellow).to_string()
        };

        colored_line.into()
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Validator for ShellHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //execute!(stdout(), EnterAlternateScreen)?;
    let config = Config::builder().color_mode(ColorMode::Enabled).build();

    let h = ShellHelper {
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        validator: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));

    let history_path = "/tmp/.shell_history";

    let _ = dashboard::print_dashboard();

    match rl.load_history(history_path) {
        Ok(_) => {}
        Err(ReadlineError::Io(_)) => {
            fs::File::create(history_path)?;
        }
        Err(err) => {
            eprintln!(
                "{}",
                style(format!("Gobbleshell: Error loading history: {}", err)).with(Color::Red)
            );
        }
    }

    let prompt = style("> ").with(Color::Blue).bold().to_string();

    loop {
        let line = rl.readline(&prompt);

        match line {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                // Add the input to history
                rl.add_history_entry(input)?;

                let mut commands = input.trim().split(" | ").peekable();
                let mut prev_stdout = None;
                let mut children: Vec<Child> = Vec::new();

                while let Some(command) = commands.next() {
                    let mut parts = command.split_whitespace();
                    let Some(command) = parts.next() else {
                        continue;
                    };
                    let args = parts;
                    let arg: String = args.clone().collect();

                    match command {
                        "cd" => {
                            let new_dir = args.peekable().peek().map_or("/home", |x| *x);
                            let root = Path::new(new_dir);
                            if let Err(e) = env::set_current_dir(root) {
                                eprintln!("{}", style(e).with(Color::Red));
                            }

                            prev_stdout = None;
                        }
                        "exit" => {
                            println!("{}", style("Goodbye!").with(Color::Green));
                            rl.save_history(history_path)?;
                            // execute!(stdout(), LeaveAlternateScreen)?;
                            return Ok(());
                        }
                        "ai" => {
                            dotenv::dotenv().expect("Failed to read file");
                            let api_key = env::var("GEMINI_API_KEY").expect("Failed to get Key");
                            let client = Gemini::new(api_key)?;
                            let response = client
                                .generate_content()
                                .with_user_message(arg)
                                .execute()
                                .await?;
                            println!("{}", response.text());
                        }
                        command => {
                            let stdin = match prev_stdout.take() {
                                Some(output) => Stdio::from(output),
                                None => Stdio::inherit(),
                            };

                            let stdout = if commands.peek().is_some() {
                                Stdio::piped()
                            } else {
                                Stdio::inherit()
                            };

                            let child = Command::new(command)
                                .args(args)
                                .stdin(stdin)
                                .stdout(stdout)
                                .spawn();

                            match child {
                                Ok(mut child) => {
                                    prev_stdout = child.stdout.take();
                                    children.push(child);
                                }
                                Err(e) => {
                                    eprintln!("{}", style(e).with(Color::Red));
                                    break;
                                }
                            };
                        }
                    }
                }

                for mut child in children {
                    let _ = child.wait();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", style("\nUse 'exit' to quit").with(Color::Yellow));
            }
            Err(ReadlineError::Eof) => {
                println!("{}", style("\nGoodbye!").with(Color::Green));
                break;
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("shell: Error: {:?}", e)).with(Color::Red)
                );
            }
        }
    }

    rl.save_history(history_path)?;
    Ok(())
}
