use self::{app_state::AppState, dev::DevCommands, info_file::InfoFile};
use crate::project::RustAnalyzerProject;
use anyhow::{bail, Context, Result};
use app_state::StateFileStatus;
use clap::{Parser, Subcommand};
use std::{
    io::{self, IsTerminal, Write},
    path::Path,
    process::ExitCode,
};
use term::{clear_terminal, press_enter_prompt};

mod app_state;
mod cargo_toml;
mod cmd;
mod dev;
mod embedded;
mod exercise;
mod info_file;
mod init;
mod list;
mod project;
mod run;
mod term;
mod watch;

const CURRENT_FORMAT_VERSION: u8 = 1;

/// Rustlings is a collection of small exercises to get you used to writing and reading Rust code
#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Subcommands>,
    /// Manually run the current exercise using `r` in the watch mode.
    /// Only use this if Rustlings fails to detect exercise file changes.
    #[arg(long)]
    manual_run: bool,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Initialize the official Rustlings exercises
    Init,
    /// Run a single exercise. Runs the next pending exercise if the exercise name is not specified
    Run {
        /// The name of the exercise
        name: Option<String>,
    },
    /// Check all the exercises, marking them as done or pending accordingly.
    CheckAll,
    /// Reset a single exercise
    Reset {
        /// The name of the exercise
        name: String,
    },
    /// Show a hint. Shows the hint of the next pending exercise if the exercise name is not specified
    Hint {
        /// The name of the exercise
        name: Option<String>,
    },
    /// Commands for developing (community) Rustlings exercises
    #[command(subcommand)]
    Dev(DevCommands),
    /// Language Server Protocol
    Lsp,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    if cfg!(not(debug_assertions)) && Path::new("dev/rustlings-repo.txt").exists() {
        bail!("{OLD_METHOD_ERR}");
    }

    'priority_cmd: {
        match args.command {
            Some(Subcommands::Init) => init::init().context("Initialization failed")?,
            Some(Subcommands::Dev(dev_command)) => dev_command.run()?,
            _ => break 'priority_cmd,
        }

        return Ok(ExitCode::SUCCESS);
    }

    if !Path::new("exercises").is_dir() {
        println!("{PRE_INIT_MSG}");
        return Ok(ExitCode::FAILURE);
    }

    let info_file = InfoFile::parse()?;

    if info_file.format_version > CURRENT_FORMAT_VERSION {
        bail!(FORMAT_VERSION_HIGHER_ERR);
    }

    let (mut app_state, state_file_status) = AppState::new(
        info_file.exercises,
        info_file.final_message.unwrap_or_default(),
    )?;

    // Show the welcome message if the state file doesn't exist yet.
    if let Some(welcome_message) = info_file.welcome_message {
        match state_file_status {
            StateFileStatus::NotRead => {
                let mut stdout = io::stdout().lock();
                clear_terminal(&mut stdout)?;

                let welcome_message = welcome_message.trim_ascii();
                write!(
                    stdout,
                    "{welcome_message}\n\n\
                     按回车键继续  "
                )?;
                press_enter_prompt(&mut stdout)?;
                clear_terminal(&mut stdout)?;
                // Flush to be able to show errors occurring before printing a newline to stdout.
                stdout.flush()?;
            }
            StateFileStatus::Read => (),
        }
    }

    match args.command {
        None => {
            if !io::stdout().is_terminal() {
                bail!("Unsupported or missing terminal/TTY");
            }

            let notify_exercise_names = if args.manual_run {
                None
            } else {
                // For the notify event handler thread.
                // Leaking is fine since the slice is used until the end of the program.
                Some(
                    &*app_state
                        .exercises()
                        .iter()
                        .map(|exercise| exercise.name.as_bytes())
                        .collect::<Vec<_>>()
                        .leak(),
                )
            };

            watch::watch(&mut app_state, notify_exercise_names)?;
        }
        Some(Subcommands::Run { name }) => {
            if let Some(name) = name {
                app_state.set_current_exercise_by_name(&name)?;
            }
            return run::run(&mut app_state);
        }
        Some(Subcommands::CheckAll) => {
            let mut stdout = io::stdout().lock();
            if let Some(first_pending_exercise_ind) = app_state.check_all_exercises(&mut stdout)? {
                if app_state.current_exercise().done {
                    app_state.set_current_exercise_ind(first_pending_exercise_ind)?;
                }

                stdout.write_all(b"\n\n")?;
                let pending = app_state.n_pending();
                if pending == 1 {
                    stdout.write_all("正在处理一个谜题: ".as_bytes())?;
                } else {
                    write!(
                        stdout,
                        "处理进度：{pending}/{}，现在正在处理: ",
                        app_state.exercises().len(),
                    )?;
                }
                app_state
                    .current_exercise()
                    .terminal_file_link(&mut stdout, app_state.emit_file_links())?;
                stdout.write_all(b"\n")?;

                return Ok(ExitCode::FAILURE);
            }

            app_state.render_final_message(&mut stdout)?;
        }
        Some(Subcommands::Reset { name }) => {
            app_state.set_current_exercise_by_name(&name)?;
            let exercise_path = app_state.reset_current_exercise()?;
            println!("The exercise {exercise_path} has been reset");
        }
        Some(Subcommands::Hint { name }) => {
            if let Some(name) = name {
                app_state.set_current_exercise_by_name(&name)?;
            }
            println!("{}", app_state.current_exercise().hint);
        }
        // Handled in an earlier match.
        Some(Subcommands::Init | Subcommands::Dev(_)) => (),
        // Handle LSP
        Some(Subcommands::Lsp) => {
            let mut project = RustAnalyzerProject::new();
            project
                .get_sysroot_src()
                .expect("Couldn't find toolchain path, do you have `rustc` installed?");
            project
                .exercises_to_json()
                .expect("Couldn't parse rustlings exercises files");

            if project.crates.is_empty() {
                println!("Failed find any exercises, make sure you're in the `rustlings` folder");
            } else if project.write_to_disk().is_err() {
                println!("Failed to write rust-project.json to disk for rust-analyzer");
            } else {
                println!("Successfully generated rust-project.json");
                println!(
                    "rust-analyzer will now parse exercises, restart your language server or editor"
                )
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

const OLD_METHOD_ERR: &str =
    "您正在尝试运行由 v6 版本或之前的 Rustlings 创建的谜题，而新版无需克隆 Rustlings 的仓库。
    参见 [`README.md`](https://github.com/rust-lang/rustlings#getting-started)";

const FORMAT_VERSION_HIGHER_ERR: &str =
    "`info.toml` 中定义的格式版本比之前的更高，您可能正在运行一个旧版的 Rustlings。
    请尝试安装一个最新版本的 Rustlings。";

const PRE_INIT_MSG: &str = r"
       欢迎使用
                 _   _ _
  _ __ _   _ ___| |_| (_)_ __   __ _ ___
 | '__| | | / __| __| | | '_ \ / _` / __|
 | |  | |_| \__ \ |_| | | | | | (_| \__ \
 |_|   \__,_|___/\__|_|_|_| |_|\__, |___/
                               |___/

当前目录下没有发现 `exercises/` 文件夹（存放着各种谜题）。
若您是第一次使用，可以运行 `rustlings init` 命令来初始化一个新的谜题文件夹。
";
