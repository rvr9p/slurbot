use tracing::info;

#[derive(poise::ChoiceParameter)]
pub enum SwearType {
    Swear = 0,
    Slur = 1,
}

#[derive(Clone)]
pub enum Swear {
    Swear(String),
    Slur(String),
}

impl Swear {
    pub async fn new(id: i32, value: String) -> Self {
        match id {
            1 => Self::Slur(value),
            _ => Self::Swear(value),
        }
    }

    pub async fn get_score(&self) -> i32 {
        match self {
            Swear::Swear(_) => 1,
            Swear::Slur(_) => 2,
        }
    }

    pub async fn get_id(&self) -> i32 {
        match self {
            Swear::Swear(_) => 0,
            Swear::Slur(_) => 1,
        }
    }

    pub async fn get_value(&self) -> String {
        match self {
            Swear::Swear(value) => value.to_string(),
            Swear::Slur(value) => value.to_string(),
        }
    }
}

pub async fn parse_swear_score(content: &String, swears: &Vec<Swear>) -> i64 {
    let stripped = [',', '.', '!', '?', '(', ')', '/'];
    let mut content = content.to_string();
    content.retain(|x| !stripped.contains(&x));
    let mut score: i64 = 0;
    for word in content.split(" ") {
        info!("PARSING WORD: {}", word);
        for swear in swears {
            if word.to_lowercase() == swear.get_value().await {
                info!(
                    "FOUND SWEAR '{}' with type {} and score {}.",
                    swear.get_value().await,
                    swear.get_id().await,
                    swear.get_score().await
                );
                score += swear.get_score().await as i64;
            }
        }
    }
    info!("Swear value for message '{}' is {}", content, &score);
    score
}
