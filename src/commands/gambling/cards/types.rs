use rand::{
    Rng, SeedableRng,
    rngs::{StdRng, ThreadRng},
    seq::SliceRandom,
};
pub enum CardFace {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

pub enum Card {
    Spades(i8),
    Hearts(i8),
    Diamonds(i8),
    Clubs(i8),
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut card_vec = Vec::new();
        for suit in [Card::Spades, Card::Hearts, Card::Diamonds, Card::Clubs] {
            for face in 1..13 {
                card_vec.push(suit(face));
            }
        }
        Self { cards: card_vec }
    }

    pub async fn shuffle(&mut self) {
        self.cards.shuffle(&mut ThreadRng::default());
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}
