use serenity::{async_trait, framework::standard::CommandResult, model::prelude::*, prelude::*};
use std::{collections::HashMap, sync::Arc};

fn emojis() -> Vec<char> {
    (0..26)
        .map(|i| std::char::from_u32('🇦' as u32 + i as u32).expect("emoji should exist"))
        .collect()
}

pub struct SurveyKey;

impl TypeMapKey for SurveyKey {
    type Value = Arc<Mutex<SurveyMap>>;
}

pub type SurveyMap = HashMap<(ChannelId, MessageId), Survey>;

pub struct Survey {
    pub question: String,
    pub answers: HashMap<char, String>,
    pub answerers: HashMap<char, usize>,
}

impl Survey {
    pub fn new(question: impl Into<String>, answers: &[impl Into<String> + Clone]) -> Self {
        let question = question.into();
        let emojis = emojis();
        let (answers, answerers) = {
            let mut tmp1 = HashMap::new();
            let mut tmp2 = HashMap::new();
            for (i, a) in answers.iter().take(26).cloned().enumerate() {
                tmp1.insert(emojis[i], a.into());
                tmp2.insert(emojis[i], 0_usize);
            }
            (tmp1, tmp2)
        };
        Self {
            question,
            answers,
            answerers,
        }
    }
}

impl std::fmt::Display for Survey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message_text = format!("**Survey:** {}\n", self.question);
        let total_answerers = self.answerers.values().sum::<usize>();

        let mut emojis: Vec<char> = self.answers.keys().copied().collect();
        emojis.sort();

        for (emoji, answer, answerers) in emojis
            .iter()
            .map(|e| (e, self.answers.get(e).unwrap(), self.answerers.get(e).unwrap()))
        {
            message_text.push(*emoji);

            if total_answerers > 0 {
                let percent = *answerers as f64 / total_answerers as f64 * 100.;
                message_text.push_str(&format!(" ({:.0}%)", percent));
            }

            message_text.push(' ');
            message_text.push_str(answer.trim_matches('"'));
            message_text.push_str(&format!(" ({} votes)", answerers));
            message_text.push('\n');
        }
        write!(f, "{}", message_text)
    }
}

#[derive(Debug)]
pub enum SurveyError {
    NoData,
    NoEmoji,
    SerenityError(SerenityError),
}

impl From<SerenityError> for SurveyError {
    fn from(e: SerenityError) -> Self {
        Self::SerenityError(e)
    }
}

impl std::fmt::Display for SurveyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tmp: String;
        write!(
            f,
            "Survey Error: {}",
            match self {
                Self::NoData => "No data",
                Self::NoEmoji => "No emoji",
                Self::SerenityError(e) => {
                    tmp = e.to_string();
                    &tmp[..]
                }
            }
        )
    }
}

#[async_trait]
pub trait SurveyManager {
    async fn update_survey_add(&self, reaction: &Reaction) -> std::result::Result<(), SurveyError>;
    async fn update_survey_rm(&self, reaction: &Reaction) -> std::result::Result<(), SurveyError>;
    async fn new_survey(&self, channel: ChannelId, survey: Survey) -> CommandResult;
}

#[async_trait]
impl SurveyManager for Context {
    async fn update_survey_add(&self, reaction: &Reaction) -> std::result::Result<(), SurveyError> {
        let key: (ChannelId, MessageId) = (reaction.channel_id, reaction.message_id);
        let mut data = self.data.write().await;
        let mut survey_map = data
            .get_mut::<SurveyKey>()
            .ok_or(SurveyError::NoData)?
            .lock()
            .await;
        let survey = survey_map.get_mut(&key).ok_or(SurveyError::NoData)?;
        let emoji = reaction
            .emoji
            .to_string()
            .chars()
            .next()
            .ok_or(SurveyError::NoEmoji)?;

        survey
            .answers
            .entry(emoji)
            .or_insert_with(|| "Other".to_owned());
        let answerers_entry = survey.answerers.entry(emoji).or_insert_with(|| 0);
        *answerers_entry += 1;

        key.0
            .edit_message(&self.http, key.1, |msg| msg.content(survey.to_string()))
            .await?;
        Ok(())
    }

    async fn update_survey_rm(&self, reaction: &Reaction) -> std::prelude::rust_2015::Result<(), SurveyError> {
        let key: (ChannelId, MessageId) = (reaction.channel_id, reaction.message_id);
        let mut data = self.data.write().await;
        let mut survey_map = data
            .get_mut::<SurveyKey>()
            .ok_or(SurveyError::NoData)?
            .lock()
            .await;
        let survey = survey_map.get_mut(&key).ok_or(SurveyError::NoData)?;
        let emoji = reaction
            .emoji
            .to_string()
            .chars()
            .next()
            .ok_or(SurveyError::NoEmoji)?;

        let answerers_entry = survey.answerers.get_mut(&emoji).unwrap();
        *answerers_entry -= 1;

        key.0
            .edit_message(&self.http, key.1, |msg| msg.content(survey.to_string()))
            .await?;
        Ok(())
    }

    async fn new_survey(&self, channel: ChannelId, survey: Survey) -> CommandResult {
        let message_text = survey.to_string();
        let survey_msg = channel.say(&self.http, &message_text).await?;

        for emoji in emojis().iter().copied().take(survey.answers.len()) {
            survey_msg
                .react(&self.http, ReactionType::Unicode(emoji.to_string()))
                .await?;
        }

        let mut survey_data = self.data.write().await;
        let survey_map = survey_data
            .get_mut::<SurveyKey>()
            .expect("error: Failed to retrieve survey map");

        survey_map
            .lock()
            .await
            .insert((channel, survey_msg.id), survey);
        Ok(())
    }
}
