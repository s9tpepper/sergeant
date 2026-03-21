use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::Widget,
};

use crate::{
    chat::ratatui_app::{
        ChatEvent,
        widgets::{Style, get_color, handle_emote, handle_text},
    },
    twitch::eventsub::deserialization::{Fragment, FragmentType},
};

pub fn has_emote(fragments: &[Fragment]) -> bool {
    fragments
        .iter()
        .find(|fragment| fragment.r#type == FragmentType::Emote)
        .is_some()
}

// TODO: Clean up all of the cloning of strings happening here.
// Turn them into string slices
fn break_apart(fragments: &Vec<Fragment>) -> Vec<Fragment> {
    let mut broken_apart: Vec<Fragment> = vec![];

    let mut word = String::new();

    #[allow(clippy::needless_range_loop)]
    for fragment in fragments {
        match fragment.r#type {
            FragmentType::Text => {
                for character in fragment.text.chars() {
                    match character {
                        ' ' => {
                            if !word.is_empty() {
                                let text = std::mem::take(&mut word);
                                broken_apart.push(Fragment {
                                    r#type: FragmentType::Text,
                                    text,
                                    cheermote: None,
                                    emote: None,
                                    mention: None,
                                });

                                word.clear();
                            }

                            broken_apart.push(Fragment {
                                r#type: FragmentType::Text,
                                text: " ".into(),
                                cheermote: None,
                                emote: None,
                                mention: None,
                            });
                        }

                        _ => word.push(character),
                    }
                }
            }
            FragmentType::Emote | FragmentType::Mention => {
                broken_apart.push(fragment.clone());
            }

            FragmentType::Cheermote => {
                info!("Found Cheermote: {fragment:?}");
            }
            FragmentType::Unknown => todo!(),
        }
    }

    if !word.is_empty() {
        broken_apart.push(Fragment {
            r#type: FragmentType::Text,
            text: word.clone(),
            cheermote: None,
            emote: None,
            mention: None,
        });
    }

    broken_apart
}

pub fn get_lines(
    fragments: &Vec<Fragment>,
    max_line_width: usize,
    name_display_space: Option<usize>,
) -> Vec<Vec<Fragment>> {
    let mut lines: Vec<Vec<Fragment>> = vec![];

    let mut current_line: Vec<Fragment> = vec![];
    let mut current_length: usize = name_display_space.unwrap_or_default();
    let mut next_length: usize;

    let mut broken_frags = break_apart(fragments);
    for index in 0..broken_frags.len() {
        let Some(fragment) = broken_frags.get(index) else {
            continue;
        };

        next_length = current_length + fragment.get_length();

        if next_length <= max_line_width {
            current_length = next_length;
        } else {
            current_length = fragment.get_length();
            lines.push(std::mem::take(&mut current_line));
        }

        current_line.push(std::mem::take(&mut broken_frags[index]));

        if current_length >= max_line_width {
            current_length = 0;
            lines.push(std::mem::take(&mut current_line));
        }
    }

    if !current_line.is_empty() {
        lines.push(std::mem::take(&mut current_line));
    }

    lines
}

impl Widget for &mut ChatEvent {
    fn render(self, writeable_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // NOTE: first_msg is not available in EventSub yet - 03/2025
        // let needs_borders = self.first_msg || is_animated;

        let line_width = writeable_area.width.saturating_sub(1);

        let lines = get_lines(&self.message.fragments, line_width as usize, None);

        let number_of_lines = lines.len();

        let y = writeable_area.height.saturating_sub(number_of_lines as u16);
        let mut cursor = Position::new(0, y);

        let style = Style {
            // TODO: Use some math on user's screen name to calculate a color based on their name
            // to default to if one is not set instead of defaulting all users to LightGreen
            fg: get_color(&self.color).unwrap_or(Color::LightGreen),
            bg: None,
        };

        for line in lines {
            line.iter().for_each(|fragment| match fragment.r#type {
                FragmentType::Text => handle_text(line_width, fragment, &style, &mut cursor, buf),

                // TODO: Implement emotes
                FragmentType::Cheermote => {}

                FragmentType::Emote => handle_emote(fragment, &mut cursor, buf),

                FragmentType::Mention => {}

                FragmentType::Unknown => {
                    unreachable!("We should never have an unknown fragment type");
                }
            });

            cursor.x = 0;
            cursor.y += 1;
        }

        self.area = writeable_area;
        self.area.height = self.area.height.saturating_sub(number_of_lines as u16);
    }
}
