use std::{
    io::{self, stdout, Stdout},
    ops::DerefMut,
    panic,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use color_eyre::config::HookBuilder;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Rect, Size},
    prelude::CrosstermBackend,
    widgets::StatefulWidget,
    Terminal,
};
use serde::{Deserialize, Serialize};
use widgets::scroll_view::{ScrollView, ScrollViewState};

use crate::{
    channel::ChannelMessages,
    twitch::eventsub::deserialization::{Badge, ChatMessageTypes, Message, NotificationEvent},
};

mod widgets;

struct RatatuiApp {
    twitch_name: String,
    receiver: Receiver<ChannelMessages>,
    // scroll_view_state: ScrollViewState,
    chat_log: Vec<ChatLogItem>,
    exit: bool,
    scrollview: ScrollView,
    scrollstate: ScrollViewState,
}

enum ChatLogItem {
    Message(ChatItem),
    MessageWithEffect(ChatItemWithEffect),
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
}

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
                area: Rect::new(0, 0, 0, 0)
            }
        } else {
            unreachable!("This should never happen");
        }
    }
}

impl RatatuiApp {
    pub fn new(twitch_name: String, receiver: Receiver<ChannelMessages>) -> Self {
        Self {
            twitch_name,
            receiver,
            exit: false,
            chat_log: vec![],
            scrollview: ScrollView::new(Size { width: 0, height: 0 }),
            scrollstate: ScrollViewState::new(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        // start TUI frontend
        let mut terminal = self.start_tui()?;

        // TODO: retrieve persisted chat log

        // Handle messages from Twitch
        while let Ok(message) = self.receiver.recv() {
            // Check if we need to break the loop
            if self.exit {
                break;
            }

            self.handle_new_message(message);

            // TODO: persist chat log

            // TODO: trigger rendering here after handling message data
            self.render(&mut terminal)?;
        }

        Ok(())
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let self_ref = Arc::new(Mutex::new(self));
        let app = self_ref.clone();
        // let appp = self_ref.clone();

        terminal.draw(|frame| {
            let mut app_lock = app.lock().unwrap();

            let stateful_widget: &mut RatatuiApp = app_lock.deref_mut();
            let mut state = stateful_widget.scrollstate;
            frame.render_stateful_widget(stateful_widget, frame.area(), &mut state);
            drop(app_lock);

            // TODO: Need to fix the scrollview, it is not scrolling/rendering
            // let mut app_lock = app.lock().unwrap();
            // state.scroll_to_bottom();
            // let stateful_widget: &mut RatatuiApp = app_lock.deref_mut();
            // stateful_widget
            //     .scrollview
            //     .render(frame.area(), frame.buffer_mut(), &mut state);
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
        }
    }

    fn ad_break(&self, message: &str) {
        todo!("ad break not implemented")
    }

    fn clear_messages(&self, user: &str) {
        todo!("clear messages not implemented")
    }

    fn redeem_refund(&self, message: &str, cmd_output: &str) {
        todo!("redeem refund not implemented")
    }

    fn automatic_reward_redeem(&mut self, message: Box<NotificationEvent>) {
        let notification = *message;

        if let NotificationEvent::ChannelPointsCustomRewardRedemptionAdd { .. } = notification {
            let chat_item: ChatItem = notification.into();

            // NOTE: https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-bits-use-event
            // This might actually come from Channel Bits Use Event, need to test how to get the
            // message effect id, either through ChannelBitsUseEvent or through ChannelPointsCustomRewardRedemptionAdd
            //
            // TODO: Figure out what effect type this is and add it to ChatItemWithEffect struct
            let chat_item_with_effect: ChatItemWithEffect = ChatItemWithEffect { chat_item };

            self.chat_log
                .push(ChatLogItem::MessageWithEffect(chat_item_with_effect));
        }
    }

    fn chat_message(&mut self, message: Box<NotificationEvent>) {
        let notification = *message;
        let chat_message: ChatItem = notification.into();

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

    fn bot_announcement(&self, message: &str) {
        todo!("bot announcement not implemented")
    }

    fn start_tui(&mut self) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).expect("No TUI");

        execute!(stdout(), EnterAlternateScreen)?;

        enable_raw_mode()?;

        // TODO: Add the chat log persistence
        // let _ = self.restore_chat_log();

        // TODO: Make a better way to handle test messages for testing the UI
        // NOTE: Test messages can go here for now
        // let test_raid_message = get_raid_message();
        // self.chat_log.insert(0, test_raid_message);

        self.render(&mut terminal);

        Ok(terminal)
    }
}

pub fn ratatui(twitch_name: String, tui_receiver: Receiver<ChannelMessages>) -> anyhow::Result<()> {
    install_hooks()?;

    let mut ratatui_app = RatatuiApp::new(twitch_name, tui_receiver);
    ratatui_app.run();

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
