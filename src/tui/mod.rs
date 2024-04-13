use color_eyre::config::HookBuilder;
use color_eyre::eyre;

use ratatui::prelude::*;

use std::io::{self, stdout, Stdout};
use std::sync::mpsc::Receiver;
use std::{error::Error, fs};
use std::{panic, time};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::{execute, terminal::*};
use ratatui::backend::CrosstermBackend;
// use ratatui::{backend::TestBackend, prelude::*};

use color_eyre::{eyre::Result, eyre::WrapErr};

use crate::tui;
use crate::twitch::parse::Text;
use crate::twitch::parse::{Emote, RedeemMessage};
use crate::twitch::pubsub::SubMessage;
use crate::twitch::ChannelMessages;
use crate::{
    twitch::{
        irc::TwitchIRC,
        parse::{ChatMessage, TwitchMessage},
    },
    utils::get_data_directory,
};

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Default)]
pub struct App {
    chat_log: Vec<ChannelMessages>,
    exit: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Text(Text),
    Emote(Emote),
}

#[derive(Debug, PartialEq)]
pub enum MessageParts {
    Text(Vec<Symbol>),
    Emote(Emote),
}

impl Clone for MessageParts {
    fn clone(&self) -> Self {
        match self {
            MessageParts::Text(symbols) => MessageParts::Text(symbols.clone()),
            MessageParts::Emote(emote) => MessageParts::Emote(emote.clone()),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            chat_log: vec![],
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui, rx: Receiver<ChannelMessages>) -> Result<()> {
        while !self.exit {
            if let Ok(message) = rx.try_recv() {
                match message {
                    ChannelMessages::TwitchMessage(message) => {
                        self.chat_log.insert(0, ChannelMessages::TwitchMessage(message));
                        self.truncate();
                    }

                    ChannelMessages::MessageData(message) => match message.data {
                        SubMessage::Points(points_message) => {
                            let message = format!(
                                "{} redeemed {} for {}",
                                points_message.redemption.user.display_name,
                                points_message.redemption.reward.title,
                                points_message.redemption.reward.cost
                            );

                            let rm = RedeemMessage { message, area: None };
                            let redeem_message = TwitchMessage::RedeemMessage { message: rm };
                            self.chat_log.insert(0, ChannelMessages::TwitchMessage(redeem_message));
                        }
                        SubMessage::Sub(_) => todo!(),
                        SubMessage::Bits(_) => todo!(),
                    },
                    ChannelMessages::Announcement(_) => todo!(),
                }
            }

            terminal.draw(|frame| self.render(frame))?;

            self.handle_events()
                .wrap_err("handle events failed, can add additional details here")?;
        }

        Ok(())
    }

    fn truncate(&mut self) {
        if self.chat_log.len() > 100 {
            self.chat_log.remove(0);
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<()> {
        // NOTE: This is a blocking call, so it will wait until an event happens,
        // this should be replaced with event::poll() in a real application.
        let available = event::poll(time::Duration::from_millis(0))?;
        if available {
            match event::read()? {
                // NOTE: it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                    .handle_key_event(key_event)
                    .wrap_err_with(|| format!("Failed to handle key event: {:?}", key_event.code)),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if let KeyCode::Char('q') = key_event.code {
            self.exit()
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut available_area = area;
        self.chat_log.iter_mut().for_each(|message| {
            let message_area = match message {
                ChannelMessages::TwitchMessage(message) => match message {
                    TwitchMessage::PrivMessage { message } => {
                        // println!("available area: {:?}", available_area);

                        message.render(available_area, buf);

                        message.area
                    }

                    TwitchMessage::RedeemMessage { message } => {
                        // println!("available area: {:?}", available_area);

                        message.render(available_area, buf);

                        message.area
                    }

                    // TwitchMessage::RedeemMessage { message } => message.render(),
                    TwitchMessage::RaidMessage { .. } => Some(Rect::new(0, 0, 0, 0)),

                    _ => Some(Rect::new(0, 0, 0, 0)),
                },

                ChannelMessages::MessageData { .. } => Some(Rect::new(0, 0, 0, 0)),

                _ => Some(Rect::new(0, 0, 0, 0)),
            };

            if let Some(message_area) = message_area {
                available_area.height = available_area.height.saturating_sub(message_area.height);
            }
        });
    }
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error: &(dyn std::error::Error + 'static)| {
        tui::restore().unwrap();
        eyre_hook(error)
    }))?;

    Ok(())
}

// NOTE: Use this for testing
// TODO: Figure out how to swap this when debugging or running tests
//
// pub type Tui = Terminal<TestBackend>;

pub fn output(message: TwitchMessage, client: &mut TwitchIRC) {
    if let TwitchMessage::PrivMessage { message } = message {
        print_message(&message, client);
    }
}

pub fn get_list_commands() -> Result<Vec<String>, Box<dyn Error>> {
    get_list("chat_commands")
}

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
    // Terminal::new(TestBackend::new(320, 240))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn get_list(directory: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let command_path = get_data_directory(Some(directory))?;
    let mut commands = vec![];
    let dir_entries = fs::read_dir(command_path)?;

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name();
            if let Some(file_name) = file_name {
                commands.push(file_name.to_string_lossy().to_string());
            }
        }
    }

    Ok(commands)
}

fn check_for_chat_commands(message: &str, client: &mut TwitchIRC) {
    let commands_list = get_list_commands();
    if let Ok(list) = &commands_list {
        for item in list {
            let command = format!("!{}", item);
            if message == command {
                let _ = output_chat_command(item, client);
            }
        }
    }
}

fn output_chat_command(command: &str, client: &mut TwitchIRC) -> Result<(), Box<dyn Error>> {
    let mut data_dir = get_data_directory(Some("chat_commands"))?;
    data_dir.push(command);

    let message = fs::read_to_string(data_dir)?;

    client.send_privmsg(format!("[bot] {}", message).as_str());

    Ok(())
}

fn print_message(message: &ChatMessage, client: &mut TwitchIRC) {
    // let (r, g, b) = get_nickname_color(&message.color);
    let nickname = &message.nickname;

    let nick = nickname;
    let final_message = format!("{nick}: {}", message.message.trim());

    if message.first_msg {
        let first_time_msg = "✨ First Time Chat:".to_string();
        println!("{}", first_time_msg);
    }

    println!("{final_message}");

    check_for_chat_commands(&message.message, client);
}