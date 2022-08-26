use rand::{thread_rng, seq::SliceRandom};
use std::{io, io::Write};

struct Card {
    rank: i8,
    suit: i8,
    flip: bool,
    face: String,
    back: String
}

struct Pile {
    cards: Vec<Card>,
    empty: String
}

impl Pile {
    fn move_cards(&mut self, pile: &mut Pile, num: usize) {
        if num > self.cards.len() { return; }

        let index = self.cards.len() - num;
        let mut temp: Vec<Card> = self.cards.drain(index..).collect();

        pile.cards.append(&mut temp);
    }

    fn move_cards_reverse(&mut self, pile: &mut Pile, num: usize) {
        if num > self.cards.len() { return; }

        for _ in 0..num {
            self.move_cards(pile, 1);
        }
    }
}

struct State {
    draw: i8,
    score: i16,

    stock: Pile,
    waste: Pile,

    tableau: Vec<Pile>,
    foundations: Vec<Pile>
}

fn main() {
    let mut states = Vec::new();
    initialize_table(&mut states);

    println!("\n\n");

    loop {
        let mut input = String::new();
        print_table(states.last().unwrap(), "> ");
        io::stdin().read_line(&mut input).unwrap();
    }
}

fn initialize_table(states: &mut Vec<State>) {
    const RANKS: [&str; 13] = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    const SUITS: [&str; 4] = ["♥", "♦", "♠", "♣"];

    let newline = "\x1b[1B\x1b[13D";

    let mut empty = String::new();
    let mut back = String::new();

    empty += &format!("┌─         ─┐{}", newline);
    for _ in 0..7 { empty += &format!("{}{}", repeat_char(' ', 13), newline); }
    empty += &format!("└─         ─┘");

    back += &format!("┌───────────┐{}", newline);
    for _ in 0..7 { back += &format!("│ ░░░░░░░░░ │{}", newline); }
    back += &format!("└───────────┘");

    let mut stock = Pile { cards: Vec::new(), empty: empty.clone() };
    let waste = Pile { cards: Vec::new(), empty: empty.clone() };

    for suit in 0..4 {
        for rank in 0..13 {
            let mut card = Card {
                rank,
                suit,
                flip: false,
                face: String::new(),
                back: back.clone()
            };

            let mut face = card.face;

            let color: &str = if suit < 2 { "\x1b[1;31m" } else { "\x1b[1;90m" };
            let reset: &str = "\x1b[0m";

            let rank: &str = RANKS[card.rank as usize];
            let suit: &str = SUITS[card.suit as usize];

            let formatted_top_rank: String = format!("{} ", rank);
            let formatted_bottom_rank: String = format!(" {}", rank);

            let top_rank: &str = if card.rank == 9 { rank } else { &formatted_top_rank };
            let bottom_rank: &str = if card.rank == 9 { rank } else { &formatted_bottom_rank };

            face += &format!("┌───────────┐{}", newline);
            face += &format!("│ {}{}      {}{} │{}", color, top_rank, suit, reset, newline);
            face += &format!("│ {}         {} │{}", color, reset, newline);
            face += &format!("│ {}         {} │{}", color, reset, newline);
            face += &format!("│ {}    {}    {} │{}", color, suit, reset, newline);
            face += &format!("│ {}         {} │{}", color, reset, newline);
            face += &format!("│ {}         {} │{}", color, reset, newline);
            face += &format!("│ {}{}      {}{} │{}", color, suit, bottom_rank, reset, newline);
            face += &format!("└───────────┘");

            card.face = face;
            stock.cards.push(card);
        }
    }

    stock.cards.shuffle(&mut thread_rng());

    let mut tableau = Vec::new();
    let mut foundations = Vec::new();

    for num in 0..7 {
        let mut column = Pile { cards: Vec::new(), empty: empty.clone() };

        stock.move_cards_reverse(&mut column, num + 1);
        column.cards.last_mut().unwrap().flip = true;

        tableau.push(column);
    }

    for _ in 0..4 {
        let foundation = Pile { cards: Vec::new(), empty: empty.clone() };
        foundations.push(foundation);
    }

    let state = State {
        draw: 1,
        score: 0,
        stock,
        waste,
        tableau,
        foundations
    };

    states.push(state);
}

fn print_table(state: &State, prompt: &str) {
    let max_lines = get_max_lines(state);

    print!("\x1b[2A");

    for _ in 0..max_lines {
        println!("\x1b[2K");
    }

    print!("\x1b[0J\x1b[{}A", max_lines);

    print_prompt(state, prompt);
    print!("\x1b[3E");
    print_stock(state);
    print!("\x1b[10A\x1b[1C");
    print_waste(state);
    print!("\x1b[10A\x1b[1C");
    print_foundations(state);
    print!("\x1b[2E");
    print_tableau(state);

    print!("\x1b[u");
    io::stdout().flush().unwrap();
}

fn print_prompt(state: &State, prompt: &str) {
    let score_len = state.score.to_string().len() as i8;
    let prompt_len = prompt.len() as i8;

    let remaining_width = 83 - score_len - prompt_len;

    let score_dashes = repeat_char('─', score_len);
    let prompt_dashes = repeat_char('─', prompt_len);
    let remaining_dashes = repeat_char('─', remaining_width);
    let remaining_spaces = repeat_char(' ', remaining_width);

    print!("┌────────{}─┬─{}{}─┐\x1b[1E", score_dashes, prompt_dashes, remaining_dashes);
    print!("│ Score: {} │ {}{} │\x1b[1E", state.score, prompt, remaining_spaces);
    print!("└────────{}─┴─{}{}─┘", score_dashes, prompt_dashes, remaining_dashes);

    print!("\x1b[1F\x1b[{}C\x1b[s", 12 + score_len + prompt_len);
}

fn print_stock(state: &State) {
    let num_cards = state.stock.cards.len();

    let card: &str;
    if num_cards > 0 { card = &state.stock.cards.last().unwrap().back; }
    else { card = &state.stock.empty; }

    print!("Stock\x1b[1E");
    print!("({} Cards)\x1b[1E", num_cards);
    print!("{}", card);
}

fn print_waste(state: &State) {
    let num_cards = state.waste.cards.len();
    let num_digits = num_cards.to_string().len();

    print!("Waste\x1b[1B\x1b[5D");
    print!("({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits);

    let mut num_spaces = 14;
    let mut last_card = state.waste.empty.clone();

    if num_cards > 0 {
        if state.draw == 3 && num_cards > 1 {
            let start = if num_cards - 3 > 0 { num_cards - 3 } else { 0 };
            let end = num_cards - 1;

            for card in &state.waste.cards[start..end] {
                print!("{}\x1b[8D ", card.face);
                num_spaces -= 6;

                for _ in 0..8 {
                    print!("\x1b[1A\x1b[1D ");
                }
            }
        }

        last_card = state.waste.cards.last().unwrap().face.clone();
    }

    print!("{}\x1b[{}C", last_card, num_spaces);
}

fn print_foundations(state: &State) {
    for i in 0..4 {
        let foundation = &state.foundations[i];

        let num_cards = foundation.cards.len();
        let num_digits = num_cards.to_string().len();

        print!("Foundation {}\x1b[1B\x1b[12D", i + 1);
        print!("({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits);

        if num_cards > 0 { print!("{}", foundation.cards.last().unwrap().face); }
        else { print!("{}", foundation.empty); }

        if i < 3 { print!("\x1b[10A\x1b[1C"); }
    }
}

fn print_tableau(state: &State) {
    for i in 0..7 {
        let column = &state.tableau[i];

        let num_cards = column.cards.len();
        let num_digits = num_cards.to_string().len();
        let mut num_lines = (num_cards as i8 * 2) + 9;

        print!("Column {}\x1b[1B\x1b[8D", i + 1);
        print!("({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits);

        if num_cards > 0 {
            for card in &column.cards {
                if card.flip == true { print!("{}", card.face); }
                else { print!("{}", card.back); }

                print!("\x1b[6A\x1b[13D");
            }

            print!("\x1b[6B\x1b[13C");

        } else {
            print!("{}", column.empty);
            num_lines += 2;
        }

        let difference = get_max_lines(state) - num_lines + 16;

        if i < 6 { print!("\x1b[{}A\x1b[1C", num_lines - 1); }
        else if difference > 0 { print!("\x1b[{}B", difference); }
    }
}

fn repeat_char(char: char, num: i8) -> String {
    let mut string = String::new();

    for _ in 0..num {
        string.push(char);
    }

    string
}

fn get_max_lines(state: &State) -> i8 {
    let mut num_cards = 0;

    for column in &state.tableau {
        if column.cards.len() as i8 > num_cards {
            num_cards = column.cards.len() as i8;
        }
    }

    let mut num_lines = (num_cards * 2) + 25;
    if num_lines == 25 { num_lines += 2; }

    num_lines
}
