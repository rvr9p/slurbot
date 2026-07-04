use tracing::info;

use crate::{
    Data, Error,
    db::{add_user_swears, get_swear_list},
    serenity::{Context, CreateMessage, FullEvent},
    swear::parse_swear_score,
};

pub async fn event_handler(ctx: &Context, event: &FullEvent, data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::Message { new_message } => {
            let swear_score =
                parse_swear_score(&new_message.content, &get_swear_list(&data.pool).await?).await;
            if swear_score > 0 {
                let message_builder = CreateMessage::new().content(format!(
                    "{} now has {} swears.",
                    new_message.author.name,
                    add_user_swears(
                        &data.pool,
                        new_message.author.id.get(),
                        match new_message.guild_id {
                            Some(value) => value.get(),
                            None => 0,
                        },
                        swear_score
                    )
                    .await?
                ));
                new_message
                    .channel_id
                    .send_message(&ctx.http, message_builder)
                    .await?;
            }
            Ok(())
        }
        FullEvent::ReactionAdd { add_reaction } => {
            info!("{}", add_reaction.emoji.to_string());
            return Ok(());
            if let Some(message_author_id) = add_reaction.message_author_id {
                if message_author_id == ctx.cache.current_user().id.get()
                    && add_reaction.emoji.to_string() == "x"
                {
                    if let Ok(message) = add_reaction.message(&ctx.http).await {
                        message.delete(&ctx.http).await;
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
