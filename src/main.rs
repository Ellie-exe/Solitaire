use rand::{thread_rng, seq::SliceRandom};
use std::{io, io::Write};
use std::io::Stdout;
use termion;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

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

    print!("{}", repeat_char('\n', 40));

    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    print_table(&mut stdout, states.last().unwrap(), "> ");

    loop {
        let input = stdin.next();

        if let Some(Ok(key)) = input {
            match key {
                termion::event::Key::Char('c') => { return; },
                termion::event::Key::Char('r') => { print_table(&mut stdout, states.last().unwrap(), "> "); },
                _ => {}
            }
        }
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

fn print_table(stdout: &mut RawTerminal<Stdout>, state: &State, prompt: &str) {
    let max_lines = get_max_lines(state);

    write!(stdout, "\x1b[1F").unwrap();

    for _ in 0..max_lines {
        write!(stdout, "\x1b[2K\x1b[1E").unwrap();
    }

    write!(stdout, "\x1b[0J\x1b[{}F", max_lines).unwrap();

    print_prompt(stdout, state, prompt);
    write!(stdout, "\x1b[3E").unwrap();
    print_stock(stdout, state);
    write!(stdout, "\x1b[10A\x1b[1C").unwrap();
    print_waste(stdout, state);
    write!(stdout, "\x1b[10A\x1b[1C").unwrap();
    print_foundations(stdout, state);
    write!(stdout, "\x1b[2E").unwrap();
    print_tableau(stdout, state);

    write!(stdout, "\x1b[u").unwrap();
    stdout.flush().unwrap();
}

fn print_prompt(stdout: &mut RawTerminal<Stdout>, state: &State, prompt: &str) {
    let score_len = state.score.to_string().len() as i8;
    let prompt_len = prompt.len() as i8;

    let remaining_width = 83 - score_len - prompt_len;

    let score_dashes = repeat_char('─', score_len);
    let prompt_dashes = repeat_char('─', prompt_len);
    let remaining_dashes = repeat_char('─', remaining_width);
    let remaining_spaces = repeat_char(' ', remaining_width);

    write!(stdout, "┌────────{}─┬─{}{}─┐\x1b[1E", score_dashes, prompt_dashes, remaining_dashes).unwrap();
    write!(stdout, "│ Score: {} │ {}{} │\x1b[1E", state.score, prompt, remaining_spaces).unwrap();
    write!(stdout, "└────────{}─┴─{}{}─┘", score_dashes, prompt_dashes, remaining_dashes).unwrap();

    write!(stdout, "\x1b[1F\x1b[{}C\x1b[s", 12 + score_len + prompt_len).unwrap();
}

fn print_stock(stdout: &mut RawTerminal<Stdout>, state: &State) {
    let num_cards = state.stock.cards.len();

    let card: &str;
    if num_cards > 0 { card = &state.stock.cards.last().unwrap().back; }
    else { card = &state.stock.empty; }

    write!(stdout, "Stock\x1b[1E").unwrap();
    write!(stdout, "({} Cards)\x1b[1E", num_cards).unwrap();
    write!(stdout, "{}", card).unwrap();
}

fn print_waste(stdout: &mut RawTerminal<Stdout>, state: &State) {
    let num_cards = state.waste.cards.len();
    let num_digits = num_cards.to_string().len();

    write!(stdout, "Waste\x1b[1B\x1b[5D").unwrap();
    write!(stdout, "({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits).unwrap();

    let mut num_spaces = 14;
    let mut last_card = state.waste.empty.clone();

    if num_cards > 0 {
        if state.draw == 3 && num_cards > 1 {
            let start = if num_cards - 3 > 0 { num_cards - 3 } else { 0 };
            let end = num_cards - 1;

            for card in &state.waste.cards[start..end] {
                write!(stdout, "{}\x1b[8D ", card.face).unwrap();
                num_spaces -= 6;

                for _ in 0..8 {
                    write!(stdout, "\x1b[1A\x1b[1D ").unwrap();
                }
            }
        }

        last_card = state.waste.cards.last().unwrap().face.clone();
    }

    write!(stdout, "{}\x1b[{}C", last_card, num_spaces).unwrap();
}

fn print_foundations(stdout: &mut RawTerminal<Stdout>, state: &State) {
    for i in 0..4 {
        let foundation = &state.foundations[i];

        let num_cards = foundation.cards.len();
        let num_digits = num_cards.to_string().len();

        write!(stdout, "Foundation {}\x1b[1B\x1b[12D", i + 1).unwrap();
        write!(stdout, "({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits).unwrap();

        if num_cards > 0 { write!(stdout, "{}", foundation.cards.last().unwrap().face).unwrap(); }
        else { write!(stdout, "{}", foundation.empty).unwrap(); }

        if i < 3 { write!(stdout, "\x1b[10A\x1b[1C").unwrap(); }
    }
}

fn print_tableau(stdout: &mut RawTerminal<Stdout>, state: &State) {
    for i in 0..7 {
        let column = &state.tableau[i];

        let num_cards = column.cards.len();
        let num_digits = num_cards.to_string().len();
        let mut num_lines = (num_cards as i8 * 2) + 9;

        write!(stdout, "Column {}\x1b[1B\x1b[8D", i + 1).unwrap();
        write!(stdout, "({} Cards)\x1b[1B\x1b[{}D", num_cards, 8 + num_digits).unwrap();

        if num_cards > 0 {
            for card in &column.cards {
                if card.flip == true { write!(stdout, "{}", card.face).unwrap(); }
                else { write!(stdout, "{}", card.back).unwrap(); }

                write!(stdout, "\x1b[6A\x1b[13D").unwrap();
            }

            write!(stdout, "\x1b[6B\x1b[13C").unwrap();

        } else {
            write!(stdout, "{}", column.empty).unwrap();
            num_lines += 2;
        }

        let difference = get_max_lines(state) - num_lines + 16;

        if i < 6 { write!(stdout, "\x1b[{}A\x1b[1C", num_lines - 1).unwrap(); }
        else if difference > 0 { write!(stdout, "\x1b[{}B", difference).unwrap(); }
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
