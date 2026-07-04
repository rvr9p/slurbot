use std::{fmt, time::Duration};

use futures::StreamExt;
use poise::CreateReply;

use crate::{
    commands::gambling::cards::types::{Card, Deck},
    serenity::{
        ButtonStyle, Colour, ComponentInteractionCollector, CreateActionRow, CreateButton,
        CreateEmbed, CreateEmbedAuthor, CreateInteractionResponse, User,
    },
    types::{Context, Error},
};

struct BlackjackCard {
    card: Card,
    hidden: bool,
}

enum Visibility {
    Hidden,
    Shown,
}

impl BlackjackCard {
    fn new(card: Card) -> Self {
        Self {
            card,
            hidden: false,
        }
    }
    fn set_visibility(&mut self, visibility: Visibility) {
        self.hidden = match visibility {
            Visibility::Hidden => true,
            Visibility::Shown => false,
        }
    }
}

struct BlackjackHand {
    cards: Vec<BlackjackCard>,
}

struct BlackjackScore {
    base_score: i16,
    prefix: Option<String>,
}

impl BlackjackHand {
    fn get_score(&self, include_hidden: bool) -> BlackjackScore {
        let mut score: BlackjackScore = BlackjackScore {
            base_score: 0,
            prefix: None,
        };
        let mut aces = 0;
        for card in &self.cards {
            if card.hidden {
                score.prefix = Some(">".to_string());
                if !include_hidden {
                    continue;
                }
            }
            let card_value = match card.card {
                Card::Spades(val) => val,
                Card::Hearts(val) => val,
                Card::Diamonds(val) => val,
                Card::Clubs(val) => val,
            };

            score.base_score += match card_value {
                1 => {
                    aces += 1;
                    0
                }
                11 | 12 | 13 => 10,
                _ => card_value as i16,
            }
        }
        for _ in 0..aces {
            if (score.base_score + 11) > 21 {
                score.base_score += 1;
            } else {
                score.base_score += 11;
            }
        }
        score
    }
}

impl fmt::Display for BlackjackScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.prefix.clone().unwrap_or("".to_string()),
            self.base_score
        )
    }
}

impl fmt::Display for BlackjackCard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.hidden {
            return write!(f, "? ?");
        }
        match self.card {
            Card::Spades(val) => write!(f, ":spades: {val}"),
            Card::Hearts(val) => write!(f, ":hearts: {val}"),
            Card::Diamonds(val) => write!(f, ":diamonds: {val}"),
            Card::Clubs(val) => write!(f, ":clubs: {val}"),
        }
    }
}

impl fmt::Display for BlackjackHand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.cards
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

enum StateReason {
    Bust,
    Blackjack,
}

enum GameState {
    Unfinished,
    Win(StateReason),
    Tie(StateReason),
    Loss(StateReason),
}

impl GameState {
    fn from(game: &BlackjackGame) -> GameState {
        let dealer_bust = game.dealer_hand.get_score(true).base_score > 21;
        let user_bust = game.user_hand.get_score(true).base_score > 21;
        if user_bust {
            
        }
        if !game.running {
            match 
        }
    
        GameState::Unfinished
    }
}

struct BlackjackGame {
    deck: Deck,
    dealer_hand: BlackjackHand,
    user_hand: BlackjackHand,
    running: bool,
    game_state: GameState,
}

enum BlackjackMoves {
    Hit,
    Stand,
}

impl BlackjackGame {
    fn new() -> Self {
        Self {
            deck: Deck::new(),
            dealer_hand: BlackjackHand { cards: Vec::new() },
            user_hand: BlackjackHand { cards: Vec::new() },
            running: true,
            game_state: GameState::Unfinished,
        }
    }

    async fn setup(&mut self) -> Result<(), Error> {
        self.deck.shuffle().await;
        for _ in 0..2 {
            self.user_hand
                .cards
                .push(BlackjackCard::new(self.deck.draw().unwrap()))
        }
        for _ in 0..2 {
            self.dealer_hand
                .cards
                .push(BlackjackCard::new(self.deck.draw().unwrap()))
        }
        self.dealer_hand
            .cards
            .get_mut(1)
            .unwrap()
            .set_visibility(Visibility::Hidden);
        Ok(())
    }
    async fn hit(&mut self) -> Result<(), Error> {
        self.user_hand
            .cards
            .push(BlackjackCard::new(self.deck.draw().unwrap()));
        if self.dealer_hand.get_score(true).base_score <= 17 {
            self.dealer_hand
                .cards
                .push(BlackjackCard::new(self.deck.draw().unwrap()))
        }
        Ok(())
    }
    async fn end_game(&mut self) {
        self.game_state = GameState::from(&self);
    }
    async fn stand(&mut self) -> Result<(), Error> {}
}

async fn construct_blackjack_embed(
    game: &BlackjackGame,
    author: &User,
) -> Result<CreateEmbed, Error> {
    Ok(CreateEmbed::new()
        .author(
            CreateEmbedAuthor::new(&author.name)
                .icon_url(author.avatar_url().unwrap_or("".to_string())),
        )
        .color(Colour::BLUE)
        .title(":spades: | Blackjack")
        .field(
            format!("Dealer ({})", game.dealer_hand.get_score(false)),
            game.dealer_hand.to_string(),
            false,
        )
        .field(
            format!("{} ({})", author.name, game.user_hand.get_score(false)),
            game.user_hand.to_string(),
            false,
        ))
}

async fn construct_action_row() -> Result<CreateActionRow, Error> {
    let buttons = vec![
        CreateButton::new("action_hit")
            .label("Hit")
            .style(ButtonStyle::Primary),
        CreateButton::new("action_stand")
            .label("Stand")
            .style(ButtonStyle::Primary),
    ];

    Ok(CreateActionRow::Buttons(buttons))
}

#[poise::command(slash_command, description_localized("en-US", "Play Blackjack"))]
pub async fn blackjack(ctx: Context<'_>, #[description = "Bet"] bet: u64) -> Result<(), Error> {
    let mut game = BlackjackGame::new();
    game.setup().await?;
    let embed = construct_blackjack_embed(&game, ctx.author()).await?;
    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![construct_action_row().await?]),
    )
    .await?;

    let mut collector = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(120))
        .stream();

    while game.running
        && let Some(press) = collector.next().await
    {
        if let Some(hole_card) = game.dealer_hand.cards.get_mut(1) {
            hole_card.set_visibility(Visibility::Shown);
        }

        match press.data.custom_id.as_str() {
            "action_hit" => game.hit().await?,
            "action_stand" => game.stand().await?,
            _ => continue,
        }
    }
    if game.running {}
    Ok(())
}
