use crate::scrollview::scroll_view::ScrollView;
use crate::scrollview::state::ScrollViewState;

use color_eyre::config::HookBuilder;
use color_eyre::eyre;

use ratatui::prelude::*;

use std::io::{self, stdout, Stdout};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
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
use crate::twitch::pubsub::{send_to_error_log, SubMessage};
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
    scroll_view_state: ScrollViewState,
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
        let scroll_view_state = ScrollViewState::new();

        App {
            scroll_view_state,
            chat_log: vec![],
            exit: false,
        }
    }

    pub fn run(&mut self, rx: Receiver<ChannelMessages>, socket_tx: Sender<ChannelMessages>) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).expect("No TUI");

        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let _ = self.restore_chat_log();

        while !self.exit {
            if let Ok(message) = rx.try_recv() {
                match &message {
                    ChannelMessages::TwitchMessage(message) => {
                        match message {
                            TwitchMessage::ClearMessage { message } => {
                                self.chat_log.retain(|msg| match msg {
                                    ChannelMessages::TwitchMessage(TwitchMessage::PrivMessage { message: msg }) => {
                                        msg.id != message.message_id
                                    }
                                    _ => true,
                                });
                            }

                            // NOTE: This message comes from EventSub channel
                            TwitchMessage::ClearMessageByUser { message } => {
                                self.chat_log.retain(|msg| match msg {
                                    ChannelMessages::TwitchMessage(TwitchMessage::PrivMessage { message: msg }) => {
                                        msg.nickname.to_lowercase() != message.display_name.to_lowercase()
                                    }
                                    _ => true,
                                });
                            }

                            _ => {
                                // This line removes the artifacts from behind emotes, but is now causing
                                // the chat to flicker when a new message is received
                                let _ = terminal.backend_mut().clear_region(backend::ClearType::All);

                                self.chat_log.insert(0, ChannelMessages::TwitchMessage(message.clone()));
                                self.truncate();
                            }
                        }
                    }

                    ChannelMessages::MessageData(message) => match &message.data {
                        SubMessage::Points(points_message) => {
                            let message = format!(
                                "{} redeemed {} for {}",
                                points_message.redemption.user.display_name,
                                points_message.redemption.reward.title,
                                points_message.redemption.reward.cost
                            );

                            let rm = RedeemMessage {
                                message,
                                area: None,
                                color: None,
                            };
                            let redeem_message = TwitchMessage::RedeemMessage { message: rm };
                            self.chat_log.insert(0, ChannelMessages::TwitchMessage(redeem_message));
                        }

                        // TODO: Implement Sub and Bits messages
                        SubMessage::Sub(_) => {}
                        SubMessage::Bits(_) => {}
                    },

                    // noop here
                    ChannelMessages::Announcement(_) => {}

                    ChannelMessages::Notifications(subscription_event) => {
                        if let Some(notice_type) = &subscription_event.notice_type {
                            match notice_type.as_str() {
                                "announcement" => {
                                    let mut color = String::from_str("grey").unwrap();
                                    if let Some(announcement) = &subscription_event.announcement {
                                        color = announcement.color.to_string();
                                    }
                                    let rgb = color_name::Color::val().by_string(color);
                                    let message_color = rgb.unwrap_or((128, 128, 128).into());

                                    if let Some(message) = &subscription_event.message {
                                        let redeem_message = RedeemMessage {
                                            message: message.text.clone(),
                                            area: None,
                                            color: Some(message_color.into()),
                                        };

                                        let redeem_message = TwitchMessage::RedeemMessage {
                                            message: redeem_message,
                                        };
                                        self.chat_log.insert(0, ChannelMessages::TwitchMessage(redeem_message));
                                    }
                                }

                                &_ => {}
                            }
                        }
                    }
                }

                let _ = self.persist_chat_log();
                let socket_tx_send_result = socket_tx.send(message);
                if let Err(send_error) = socket_tx_send_result {
                    send_to_error_log(
                        "websocket: Could not transmit message to channel".to_string(),
                        send_error.to_string(),
                    )
                } else {
                    send_to_error_log(
                        "websocket: Successfully transmitted message to channel".to_string(),
                        "Success".to_string(),
                    );
                }
            }

            terminal.draw(|frame| self.render(frame))?;
        }

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn restore_chat_log(&mut self) -> Result<(), Box<dyn Error>> {
        let target_dir = "chat_log";
        let mut chat_log_path = get_data_directory(Some(target_dir))?;
        chat_log_path.push("log.txt");

        if chat_log_path.is_file() {
            let chat_log_str = fs::read_to_string(&chat_log_path).expect("");
            if let Ok(chat_log) = serde_json::from_str(&chat_log_str) {
                self.chat_log = chat_log;
            }
        }

        Ok(())
    }

    fn persist_chat_log(&self) -> Result<(), Box<dyn Error>> {
        let json_string = serde_json::to_string(&self.chat_log).unwrap_or_default();

        let target_dir = "chat_log";
        let mut chat_log_path = get_data_directory(Some(target_dir))?;

        if !chat_log_path.exists() {
            fs::create_dir_all(&chat_log_path)?;
        }

        if !json_string.is_empty() {
            chat_log_path.push("log.txt");
            fs::write(chat_log_path, json_string)?;
        }

        Ok(())
    }

    fn truncate(&mut self) {
        if self.chat_log.len() > 100 {
            self.chat_log.remove(self.chat_log.len() - 1);
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.reset();

        let mut scroll_view_state = self.scroll_view_state;
        let mut scroller = Scroller::new(self);
        scroller.render(area, buf, &mut scroll_view_state);
        let _ = scroller.handle_events();
    }
}

struct Scroller<'a> {
    app: &'a mut App,
}

impl<'a> Scroller<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Scroller { app }
    }

    pub fn handle_events(&mut self) -> Result<()> {
        let available = event::poll(time::Duration::from_millis(16))?;
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
        match key_event.code {
            KeyCode::Char('q') => self.app.exit(),
            KeyCode::Char('j') => self.app.scroll_view_state.scroll_down(),
            KeyCode::Char('k') => self.app.scroll_view_state.scroll_up(),
            KeyCode::Char('f') => self.app.scroll_view_state.scroll_page_down(),
            KeyCode::Char('b') => self.app.scroll_view_state.scroll_page_up(),
            KeyCode::Char('g') => self.app.scroll_view_state.scroll_to_top(),
            KeyCode::Char('G') => self.app.scroll_view_state.scroll_to_bottom(),
            _ => {}
        }

        Ok(())
    }
}

impl<'a> StatefulWidget for &mut Scroller<'a> {
    type State = ScrollViewState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let content_size = layout::Size {
            // Subtract one to avoid getting horizontal scrollbar from tui-scrollview
            width: area.width.saturating_sub(1),
            height: area.height * 2,
        };

        let mut scroll_view = ScrollView::new(content_size);

        let mut available_area = area;
        available_area.height = content_size.height;

        if self.app.chat_log.is_empty() {
            self.app.scroll_view_state.scroll_to_bottom();

            // NOTE: Push messages here to test with
            //
            // let chat_message = TwitchMessage::PrivMessage {
            //     message: ChatMessage {
            //         id: "1234".to_string(),
            //         nickname: "some_person".to_string(),
            //         color: "#FF0000".to_string(),
            //         message: "This is a test message".to_string(),
            //         first_msg: false,
            //         badges: vec![],
            //         emotes: vec![],
            //         returning_chatter: false,
            //         subscriber: false,
            //         moderator: false,
            //         channel: "some_channel".to_string(),
            //         raw: "raw message".to_string(),
            //         area: None,
            //     },
            // };
            //
            // self.app.chat_log.push(ChannelMessages::TwitchMessage(chat_message));
            //
            // let message = RaidMessage {
            //     display_name: "some_person".to_string(),
            //     user_id: "1234".to_string(),
            //     raid_notice: "1 raiders from some_person have joined!".to_string(),
            //     area: None,
            // };
            //
            // self.app
            //     .chat_log
            //     .push(ChannelMessages::TwitchMessage(TwitchMessage::RaidMessage { message }));
            //
            // let chat_message = TwitchMessage::PrivMessage {
            //     message: ChatMessage {
            //         id: "1234".to_string(),
            //         nickname: "some_person".to_string(),
            //         color: "#FF0000".to_string(),
            //         message: "This is a test message".to_string(),
            //         first_msg: false,
            //         badges: vec![],
            //         emotes: vec![],
            //         returning_chatter: false,
            //         subscriber: false,
            //         moderator: false,
            //         channel: "some_channel".to_string(),
            //         raw: "raw message".to_string(),
            //         area: None,
            //     },
            // };
            // self.app.chat_log.push(ChannelMessages::TwitchMessage(chat_message));

            // let data = SubMessage::Sub(SubscribeEvent {
            //     area: None,
            //     topic: "topic string".to_string(),
            //     message: SubscribeMessage {
            //         display_name: "some_dude".to_string(),
            //         cumulative_months: 8,
            //         streak_months: 10,
            //         context: "".to_string(), // subgift, resub
            //         sub_message: "This is a subscription message".to_string(),
            //     },
            // });
            //
            // self.app
            //     .chat_log
            //     .push(ChannelMessages::MessageData(MessageData { data }))

            // pub is_anonymous: bool,
            // pub message_type: String,
            // pub data: BitsEventData,
            //
            // BitsEventData
            // pub user_name: String,
            // pub chat_message: String,
            // pub bits_used: u64,
            // pub total_bits_used: u64,
            // pub context: String, // cheer

            // let data = SubMessage::Bits(BitsEvent {
            //     area: None,
            //     is_anonymous: false,
            //     message_type: "bits_event".to_string(),
            //     data: BitsEventData {
            //         user_name: "some_dude".to_string(),
            //         chat_message: "some cheery message".to_string(),
            //         bits_used: 500,
            //         total_bits_used: 1000,
            //         context: "cheer".to_string(),
            //     },
            // });
            //
            // self.app
            //     .chat_log
            //     .push(ChannelMessages::MessageData(MessageData { data }))
        }

        self.app.chat_log.iter_mut().for_each(|message| {
            let message_area = match message {
                ChannelMessages::TwitchMessage(message) => match message {
                    TwitchMessage::PrivMessage { message } => {
                        message.render(available_area, scroll_view.buf_mut());

                        message.area
                    }

                    TwitchMessage::RedeemMessage { message } => {
                        message.render(available_area, scroll_view.buf_mut());

                        message.area
                    }

                    TwitchMessage::RaidMessage { message } => {
                        message.render(available_area, scroll_view.buf_mut());

                        message.area
                    }

                    _ => Some(Rect::new(0, 0, 0, 0)),
                },

                ChannelMessages::MessageData(message) => match message.data {
                    SubMessage::Sub(ref mut sub_message) => {
                        sub_message.render(available_area, scroll_view.buf_mut());

                        sub_message.area
                    }

                    SubMessage::Bits(ref mut sub_message) => {
                        sub_message.render(available_area, scroll_view.buf_mut());

                        sub_message.area
                    }

                    _ => Some(Rect::new(0, 0, 0, 0)),
                },

                _ => Some(Rect::new(0, 0, 0, 0)),
            };

            if let Some(message_area) = message_area {
                available_area.height = available_area.height.saturating_sub(message_area.height);
            }
        });

        scroll_view.render(buf.area, buf, &mut self.app.scroll_view_state);
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
// pub fn init() -> io::Result<Tui> {
//     execute!(stdout(), EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     Terminal::new(CrosstermBackend::new(stdout()))
//     // Terminal::new(TestBackend::new(320, 240))
// }

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

pub fn check_for_chat_commands(message: &str, client: &mut TwitchIRC) {
    let commands_list = get_list_commands();
    if message == "!commands" {
        let available_commands = commands_list
            .unwrap()
            .iter()
            .map(|item| format!("!{}", item))
            .collect::<Vec<String>>()
            .join(", ");

        let message = format!("Available commands: {available_commands}");
        client.send_privmsg(message.as_str());

        // Send message to display since this IRC client is the one posting the message
        // it won't display if its not sent directly to the TUI client for rendering
        client.display_msg(&message);

        return;
    }

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

    client.send_privmsg(&message);

    // Send message to display since this IRC client is the one posting the message
    // it won't display if its not sent directly to the TUI client for rendering
    client.display_msg(&message);

    Ok(())
}

fn print_message(message: &ChatMessage, client: &mut TwitchIRC) {
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
