use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string, write, File},
    io::{self, stdout, BufReader, Stdout},
    ops::DerefMut,
    panic,
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{info, warn};
use ratatui::{
    layout::{Rect, Size},
    prelude::CrosstermBackend,
    style::Color,
    Terminal,
};
use serde::{Deserialize, Serialize};
use widgets::scroll_view::{ScrollView, ScrollViewState};

use crate::{
    channel::ChannelMessages,
    fs::get_data_directory,
    twitch::{
        assets::BadgeItem,
        eventsub::deserialization::{Badge, ChatMessageTypes, Fragment, FragmentType, Message, NotificationEvent},
    },
};

mod widgets;

struct RatatuiApp {
    #[allow(unused)]
    twitch_name: String,
    receiver: Receiver<ChannelMessages>,
    global_badges: HashMap<String, BadgeItem>,
    channel_badges: HashMap<String, BadgeItem>,
    chat_log: Vec<ChatLogItem>,
    exit: bool,
    scrollview: ScrollView,
    scrollstate: ScrollViewState,
    test_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum ChatLogItem {
    Message(ChatItem),
    MessageWithEffect(ChatItemWithEffect),
    Event(ChatEvent),
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatEvent {
    #[serde(skip)]
    area: Rect,
    message: Message,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatItem {
    #[serde(skip)]
    area: Rect,
    message: Message,
    color: String,
    message_type: ChatMessageTypes,
    message_id: String,
    chatter_user_name: String,
    badges: Vec<Badge>,
    badge_items: Vec<BadgeItem>,
}

// TODO: Finish this struct so it can render with effects
#[derive(Debug, Serialize, Deserialize)]
struct ChatItemWithEffect {
    chat_item: ChatItem,
    // message_effect: SomeThing
}

impl From<NotificationEvent> for ChatItem {
    fn from(value: NotificationEvent) -> Self {
        if let NotificationEvent::ChannelChatMessage {
            message,
            message_type,
            message_id,
            color,
            chatter_user_name,
            badges,
            ..

            // broadcaster_user_id,
            // broadcaster_user_name,
            // broadcaster_user_login,
            // chatter_user_id,
            // chatter_user_login,
            // cheer,
            // reply,
            // channel_points_animation_id,
            // channel_points_custom_reward_id,
            // source_broadcaster_user_id,
            // source_broadcaster_user_name,
            // source_broadcaster_user_login,
            // source_message_id,
            // source_badges,
        } = value
        {
            ChatItem {
                message,
                color,
                message_type,
                message_id,
                chatter_user_name,
                badges,
                badge_items: vec![],
                area: Rect::default()
            }
        } else {
            unreachable!("This should never happen");
        }
    }
}

impl RatatuiApp {
    pub fn new(
        twitch_name: String,
        receiver: Receiver<ChannelMessages>,
        global_badges: HashMap<String, BadgeItem>,
        channel_badges: HashMap<String, BadgeItem>,
        test_mode: bool,
    ) -> Self {
        Self {
            twitch_name,
            receiver,
            global_badges,
            channel_badges,
            test_mode,
            exit: false,
            chat_log: vec![],
            scrollview: ScrollView::new(Size { width: 0, height: 0 }),
            scrollstate: ScrollViewState::new(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        info!("ratatui_app::run()");

        // start TUI frontend
        let mut terminal = self.start_tui()?;

        if self.test_mode {
            self.load_test_messages(&mut terminal)?;
        } else {
            self.load_persisted_chat(&mut terminal)?;
        }

        // Handle messages from Twitch
        while !self.exit {
            self.handle_keyboard_events(&mut terminal)?;

            if !self.test_mode {
                if let Ok(message) = self.receiver.try_recv() {
                    // Check if we need to break the loop
                    if self.exit {
                        break;
                    }

                    // handles messages from TUI channel and adds to chat state
                    self.handle_new_message(message);

                    // persist chat log
                    self.persist_chat()?;

                    // trigger rendering here after handling message data
                    self.render(&mut terminal)?;
                }
            }
        }

        self.restore()?;

        Ok(())
    }

    /// Restore the terminal to its original state
    pub fn restore(&self) -> io::Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;

        disable_raw_mode()
    }

    fn load_persisted_chat(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let mut chat_log_dir = get_data_directory(Some("chat_log"))?;

        if !chat_log_dir.exists() {
            // ADDED
            create_dir_all(&chat_log_dir)?;
            //return Ok(());
        }

        chat_log_dir.push("log_v2.txt");

        // Creates an empty log_v2.txt if it doesn't exist
        if !chat_log_dir.exists() {
            write(&chat_log_dir, "[]")?;
        }

        let file = File::open(chat_log_dir)?;
        let reader = BufReader::new(file);
        let chat_log: Vec<ChatLogItem> = serde_json::from_reader(reader)?;

        self.chat_log = chat_log;

        self.render(terminal)?;

        Ok(())
    }

    fn persist_chat(&mut self) -> anyhow::Result<()> {
        let mut chat_log_dir = get_data_directory(Some("chat_log"))?;

        if !chat_log_dir.exists() {
            create_dir_all(&chat_log_dir)?;
        }

        let json = serde_json::to_string(&self.chat_log)?;

        if !json.is_empty() {
            chat_log_dir.push("log_v2.txt");
            write(chat_log_dir, json)?;
        }

        Ok(())
    }

    fn load_test_messages(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let test_msgs_path = Path::new("./test_messages.txt");

        let test_messages = read_to_string(test_msgs_path)?;
        self.chat_log = serde_json::from_str::<Vec<ChatLogItem>>(&test_messages)?;

        self.render(terminal)
    }

    fn handle_keyboard_events(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let available = poll(Duration::from_millis(16))?;

        if available {
            match event::read()? {
                // NOTE: it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Char('j') => self.scrollstate.scroll_down(),
                        KeyCode::Char('k') => self.scrollstate.scroll_up(),
                        KeyCode::Char('f') => self.scrollstate.scroll_page_down(),
                        KeyCode::Char('b') => self.scrollstate.scroll_page_up(),
                        KeyCode::Char('g') => self.scrollstate.scroll_to_top(),
                        KeyCode::Char('G') => self.scrollstate.scroll_to_bottom(),
                        _ => {}
                    }

                    self.render(terminal)
                }

                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let self_ref = Arc::new(Mutex::new(self));
        let app = self_ref.clone();

        terminal.draw(|frame| {
            let buffer = frame.buffer_mut();
            buffer.reset();

            // TODO: Need to fix the scrollview, it is not scrolling/rendering
            // let mut app_lock = app.lock().unwrap();
            // let stateful_widget: &mut RatatuiApp = app_lock.deref_mut();
            // let mut state = stateful_widget.scrollstate;
            // state.scroll_to_bottom();
            // stateful_widget
            //     .scrollview
            //     .render(frame.area(), frame.buffer_mut(), &mut state);
            // drop(app_lock);

            let mut app_lock = app.lock().unwrap();
            let stateful_widget: &mut RatatuiApp = app_lock.deref_mut();
            let mut state = stateful_widget.scrollstate;
            frame.render_stateful_widget(stateful_widget, frame.area(), &mut state);
            drop(app_lock);
        })?;

        Ok(())
    }

    fn handle_new_message(&mut self, message: ChannelMessages) {
        match message {
            ChannelMessages::AdBreak { message } => self.ad_break(&message),

            ChannelMessages::ClearMessagesByUser { target_user_name: user } => self.clear_messages(&user),

            ChannelMessages::RedeemRefund {
                message,
                command_output: cmd_output,
            } => self.redeem_refund(&message, &cmd_output),

            ChannelMessages::ChatMessage { message } => self.chat_message(message),

            ChannelMessages::BotAnnouncement { message } => self.bot_announcement(&message),

            ChannelMessages::AutomaticRewardRedeem { message } => self.automatic_reward_redeem(message),

            ChannelMessages::RedeemMessage { message, .. } => self.redeem_message(&message),
        }
    }

    #[allow(unused)]
    fn ad_break(&mut self, message: &str) {
        let message = Message {
            text: message.to_string(),
            fragments: vec![Fragment {
                r#type: FragmentType::Text,
                text: message.to_string(),
                cheermote: None,
                emote: None,
                mention: None,
            }],
        };

        let chat_event: ChatEvent = ChatEvent {
            area: Rect::default(),
            message,
            color: Color::Cyan.to_string(),
        };

        self.chat_log.insert(0, ChatLogItem::Event(chat_event));
    }

    #[allow(unused)]
    fn clear_messages(&self, user: &str) {
        warn!("clear messages not implemented")
    }

    #[allow(unused)]
    fn redeem_refund(&mut self, msg: &str, cmd_output: &str) {
        let message_text = format!("{msg}: {cmd_output}");
        let message = Message {
            text: message_text.to_string(),
            fragments: vec![Fragment {
                r#type: FragmentType::Text,
                text: message_text.to_string(),
                cheermote: None,
                emote: None,
                mention: None,
            }],
        };

        let chat_event: ChatEvent = ChatEvent {
            area: Rect::default(),
            message,
            color: Color::Magenta.to_string(),
        };

        self.chat_log.insert(0, ChatLogItem::Event(chat_event));
    }

    // TODO: Figure out what automatic reward redeems actually are, because the docs aren't clear
    fn automatic_reward_redeem(&mut self, message: Box<NotificationEvent>) {
        let _notification = *message;

        // NOTE: The below is in the wrong place, AutomaticRewardRedeem is not bits use or messages with
        // effects, Docs: https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchannel_points_automatic_reward_redemptionadd
        //
        // if let NotificationEvent::ChannelPointsCustomRewardRedemptionAdd { .. } = notification {
        //     let chat_item: ChatItem = notification.into();
        //
        //     // NOTE: https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-bits-use-event
        //     // This might actually come from Channel Bits Use Event, need to test how to get the
        //     // message effect id, either through ChannelBitsUseEvent or through ChannelPointsCustomRewardRedemptionAdd
        //     //
        //     self.chat_log
        //         .push(ChatLogItem::MessageWithEffect(chat_item_with_effect));
        // }
    }

    fn chat_message(&mut self, message: Box<NotificationEvent>) {
        let notification = *message;
        let mut chat_message: ChatItem = notification.into();

        chat_message.badges.iter().for_each(|badge: &Badge| {
            let badge_item = self.global_badges.get(&badge.set_id);
            match badge_item {
                Some(item) => {
                    chat_message.badge_items.push(item.clone());
                }
                None => {
                    if let Some(item) = self.channel_badges.get(&badge.set_id) {
                        chat_message.badge_items.push(item.clone());
                    }
                }
            }
        });

        // add this to the chat log vec
        self.chat_log.insert(0, ChatLogItem::Message(chat_message));

        // NOTE: This is the old render logic
        // =================================================================================
        // // This line removes the artifacts from behind emotes, but is now causing
        // // the chat to flicker when a new message is received
        // let _ = terminal.backend_mut().clear_region(backend::ClearType::All);
        //
        // let mut display_new_msg = true;
        // if let TwitchMessage::PrivMessage { message } = message {
        //     let msg = message.message.clone();
        //     let repeated = self.chat_log.iter().any(|item| match item {
        //         ChannelMessages::TwitchMessage(tm) => {
        //             if let TwitchMessage::PrivMessage { message: pm } = tm {
        //                 return pm.message == msg && pm.is_bot;
        //             }
        //
        //             false
        //         }
        //         _ => false,
        //     });
        //
        //     if message.nickname == self.twitch_name // is streamer
        //                                 && !message.is_bot // message is not from bot
        //                                 && repeated
        //     {
        //         display_new_msg = false;
        //     }
        // };

        // ===============================================================================
        // NOTE: Don't think this is necessary, it was a bandaid for trying to
        // remove duplicated messages in the chat log, those turned out to be a result
        // of the twitch message parsing logic in v1, which was updating the chat log
        // in the parsing logic
        //
        // if display_new_msg {
        //     self.chat_log.insert(0, ChannelMessages::TwitchMessage(message.clone()));
        //     self.truncate();
        //
        //     terminal.draw(|frame| self.render(frame))?;
        // }
    }

    fn redeem_message(&mut self, msg: &str) {
        let message = Message {
            text: msg.to_string(),
            fragments: vec![Fragment {
                r#type: FragmentType::Text,
                text: msg.to_string(),
                cheermote: None,
                emote: None,
                mention: None,
            }],
        };

        let chat_event: ChatEvent = ChatEvent {
            area: Rect::default(),
            message,
            color: Color::Green.to_string(),
        };

        self.chat_log.insert(0, ChatLogItem::Event(chat_event));
    }

    #[allow(unused)]
    fn bot_announcement(&self, message: &str) {
        // TODO: Implement bot announcements
        warn!("bot announcement not implemented")
    }

    fn start_tui(&mut self) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).expect("No TUI");

        execute!(stdout(), EnterAlternateScreen)?;

        enable_raw_mode()?;

        // TODO: Add restoring the persisted chat log
        // self.restore_chat_log()?;

        // TODO: Make a better way to handle test messages for testing the UI
        // NOTE: Test messages can go here for now
        // let test_raid_message = get_raid_message();
        // self.chat_log.insert(0, test_raid_message);

        self.render(&mut terminal)?;

        Ok(terminal)
    }
}

pub fn ratatui(
    twitch_name: String,
    tui_receiver: Receiver<ChannelMessages>,
    global_badges: HashMap<String, BadgeItem>,
    channel_badges: HashMap<String, BadgeItem>,
    test_mode: bool,
) -> anyhow::Result<()> {
    install_hooks()?;

    let mut ratatui_app = RatatuiApp::new(twitch_name, tui_receiver, global_badges, channel_badges, test_mode);
    ratatui_app.run()?;

    Ok(())
}

pub fn install_hooks() -> anyhow::Result<()> {
    let (panic_hook, _eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        panic_hook(panic_info);
    }));

    // // convert from a color_eyre EyreHook to a eyre ErrorHook
    // let eyre_hook = eyre_hook.into_eyre_hook();
    // eyre::set_hook(Box::new(move |error: &(dyn std::error::Error + 'static)| {
    //     restore().unwrap();
    //     eyre_hook(error)
    // }))?;

    Ok(())
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
