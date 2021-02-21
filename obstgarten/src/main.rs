use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
enum GameResult {
    Won,
    Lost,
}

enum Dice {
    Green,
    Blue,
    Red,
    Yellow,
    Basket,
    Raven,
}

impl Distribution<Dice> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dice {
        match rng.gen_range(0..6) {
            0 => Dice::Green,
            1 => Dice::Blue,
            2 => Dice::Red,
            3 => Dice::Yellow,
            4 => Dice::Basket,
            _ => Dice::Raven,
        }
    }
}

fn minus_one_or_zero(value: u32) -> u32 {
    if value > 0 {
        return value - 1;
    } else {
        return 0;
    }
}

fn play(nb_fruits: u32, nb_raven_cards: u32) -> GameResult {
    let mut current_raven_position: u32 = nb_raven_cards;

    // [greens, blues, reds, yellows]
    let mut fruits = vec![nb_fruits; 4];

    loop {
        let dice_roll: Dice = rand::random();
        match dice_roll {
            Dice::Green => fruits[0] = minus_one_or_zero(fruits[0]),
            Dice::Blue => fruits[1] = minus_one_or_zero(fruits[1]),
            Dice::Red => fruits[2] = minus_one_or_zero(fruits[2]),
            Dice::Yellow => fruits[3] = minus_one_or_zero(fruits[3]),
            Dice::Basket => {
                let (argmax, _) = fruits.iter().enumerate().max_by_key(|&(_, e)| e).unwrap();
                fruits[argmax] -= 1
            }
            Dice::Raven => current_raven_position -= 1,
        }

        if fruits.iter().sum::<u32>() == 0 {
            return GameResult::Won;
        }
        if current_raven_position == 0 {
            return GameResult::Lost;
        }
    }
}

fn main() {
    let number_games = 2000000;
    let nb_fruits = 4;
    let nb_raven_cards = 5;

    let mut nb_victories = 0;
    let mut nb_losses = 0;

    for _ in 0..number_games {
        match play(nb_fruits, nb_raven_cards) {
            GameResult::Won => nb_victories += 1,
            GameResult::Lost => nb_losses += 1,
        }
    }

    println!(
        "Likelihood winning: {}",
        nb_victories as f32 / number_games as f32
    );
    println!(
        "Likelihood loosing: {}",
        nb_losses as f32 / number_games as f32
    );
}
