use std::io::{self, Write, Result, Error, ErrorKind};
use std::str::FromStr;
use std::fmt::Display;

fn input<T, E>(message: &str, error: &str) -> T 
where 
    T: FromStr<Err = E>,
    E: Display,
    
{
    let mut u_input = String::new();
    
    loop {
    
        print!("{}: ", message);
        io::stdout().flush().expect("Failed to flush terminal.");
        
        
        let n = io::stdin().read_line(&mut u_input).expect("Input failed.");
        
        if n == 0 {
            println!("0 Bytes read. Please retry the program or ensure EOF hasn't reached.");
        }
        
        match u_input.trim().parse::<T>() {
            Ok(val) => return val,
            Err(e) => println!("Error: {} ({})", error, e),
        }
        
        u_input.clear();
    
    }
    
}

// For a random number generator on windows
#[cfg(windows)]
#[link(name = "bcrypt")]
unsafe extern "system" {
    unsafe fn BCryptGenRandom(
        hAlgorithm: *mut core::ffi::c_void,
        pbBuffer: *mut u8,
        cbBuffer: u32,
        dwFlags: u32,
    ) -> i32;
}

#[derive(Debug, Clone, Copy)]
struct Randler {
    seed: u64
}

// For good practice, add default
impl Default for Randler {
    fn default() -> Self {
        // Bro if you went out of your way to use a custom OS not derived from Linux or Windows AND are compiling this project? That's all on you
        Self::urandom_seed_init().expect("Failed to seed from /dev/urandom or bcryptprimitives. Use new(seed) instead.")
    }
}

impl Randler {
    // Creates a new instance based on a seed
    pub fn new(seed: u64) -> Self {
        if seed == 0 {
            println!("Warning! A seed of 0 will result in every random value being the same.");
        }
        Randler { seed: seed }
    }

    // Gets a seed based off of urandom
    #[cfg(unix)]
    fn get_base_random_udev() -> Result<u64> {
        use std::fs::File;
        use std::io::Read;
    
        let mut file = File::open("/dev/urandom")?;
        
        let mut buffer = [0u8; 8];
        file.read_exact(&mut buffer)?;
        
        let random_num = u64::from_ne_bytes(buffer);
        
        if random_num == 0 {
            return Ok(1);
        }
    
        Ok(random_num)
    }

    // Gets a random seed based off of bcryptprimitives
    #[cfg(windows)]
    pub fn get_base_random_udev() -> Result<u64> {
        let mut buf = [0u8; 8];
        const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

        let status = unsafe {
            BCryptGenRandom(
                core::ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };

        // NTSTATUS: success is >= 0
        if status < 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("BCryptGenRandom failed with status {status:#x}"),
            ));
        }

        let value = u64::from_le_bytes(buf);

        // Nonzero check
        if value == 0 {
            return Ok(1);
        }

        Ok(value)
    }

    // Automatically instantiates an instance based off of the seed
    pub fn urandom_seed_init() -> Result<Self> {
        Ok(Self::new(Randler::get_base_random_udev()?))
    }

    // Creates a random number based on Xorshift64
    pub fn rand(&mut self) -> u64 {
        let mut x = self.seed;
        
        x ^= x << 12;
        x ^= x >> 25;
        x ^= x << 27;
        
        self.seed = x;
        x
    }
    
    // Creates a random number within a defined range
    pub fn rand_range(&mut self, min: u64, max: u64) -> Option<u64>  {
        // Ensure input is proper
        if min > max {
            println!("Max should not be smaller than min");
            return None;
        }

        // Ensure integer overflow doesn't occur
        if min == 0 && max == u64::MAX { return Some(self.rand()); }

        // Normalize
        let range = max - min + 1;
        let limit = u64::MAX - (u64::MAX % range);

        // Prevent modulo bias
        let mut x = self.rand();
        while x >= limit {
            x = self.rand();
        }
        
        let ranged = (x % range) + min;
        
        Some(ranged)
    }
}

// Colors for the cards
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    NA
}

// So the user can input a color
impl FromStr for Color {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result< Self, Self::Err > {
        let sl = s.to_lowercase();
        match sl.as_str() {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            _ => Err( format!("{} is not an UNO standard color", s) ),
        }
    }
}

// Special cards for the cards
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
enum SpecialCard {
    PlusFour,
    ColorChange,
    PlusTwo,
    Skip,
    Reverse,
    Base
}
// One full card
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct UNOCard {
    color: Color,
    special: SpecialCard,
    number: i8,
}

impl UNOCard {
    fn new(color: Color, special: SpecialCard, number: i8) -> Self {
        UNOCard { color: color, special: special, number: number }
    }
}


#[derive(Debug, Copy, Clone)]
enum Difficulty {
    Calm,
    Aggressive,
    Skilled
}

impl FromStr for Difficulty {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result< Self, Self::Err > {
        let sl = s.to_lowercase();
        match sl.as_str() {
            "calm" => Ok(Difficulty::Calm),
            "aggressive" => Ok(Difficulty::Aggressive),
            "skilled" => Ok(Difficulty::Skilled),
            _ => Err( format!("{} is not an avaliable difficulty", s) ),
        }
    }
}

// Current game state, handling turns and reverses
#[derive(Debug, Copy, Clone)]
struct Game {
    current_player: i8,
    max_players: i8,
    direction: i8,
}

impl Game {
    fn new(c: i8, m: i8, d: i8) -> Self {
        Self { current_player: c, max_players: m, direction: d }
    }
    
    fn next_turn(&mut self) {
        self.current_player = (self.current_player + self.direction).rem_euclid(self.max_players);
    }
    
    fn reverse(&mut self) {
        self.direction *= -1;
    }
    
    fn player_number(&self) -> i8 {
        self.current_player + 1
    }
}

// Gets the name of a color from the enum
fn get_color(color: &Color) -> String {
    match color {
        Color::Red => return String::from("Red"),
        Color::Blue => return String::from("Blue"),
        Color::Yellow => return String::from("Yellow"),
        Color::Green => return String::from("Green"),
        Color::NA => return String::from("None"),
    }
}

// Formats the card message to be displayed to the user
fn format_card_message(card: &UNOCard) -> String {
    let color_str = get_color(&card.color);
    match card.special {
        SpecialCard::PlusFour => format!("Wild Draw 4 ({})", color_str),
        SpecialCard::ColorChange => format!("Wild Card ({})", color_str),
        SpecialCard::PlusTwo => format!("{} Draw 2", color_str),
        SpecialCard::Skip => format!("{} Skip", color_str),
        SpecialCard::Reverse => format!("{} Reverse", color_str),
        SpecialCard::Base => format!("{} {}", color_str, card.number),
    }
}

// Builds a full standard deck of UNO cards
fn build_deck() -> Vec<UNOCard> {
    let mut deck = Vec::with_capacity(108);
    let colors = [Color::Red, Color::Green, Color::Yellow, Color::Blue];
    let specials = [SpecialCard::Reverse, SpecialCard::Skip, SpecialCard::PlusTwo];

    for &color in &colors {
        for n in 0..=9 {
            let count = if n == 0 { 1 } else { 2 };
            for _ in 0..count {
                deck.push(UNOCard::new(color, SpecialCard::Base, n));
            }
        }
        for &spec in &specials {
            for _ in 0..2 {
                deck.push(UNOCard::new(color, spec, -1));
            }
        }
    }
    for _ in 0..4 {
        deck.push(UNOCard::new(Color::NA, SpecialCard::ColorChange, -1));
        deck.push(UNOCard::new(Color::NA, SpecialCard::PlusFour, -1));
    }
    deck
}

// Shuffles the deck
fn shuffle(deck: &mut Vec<UNOCard>, rand: &mut Randler) {
    let n = deck.len();
    for i in (1..n).rev() {
        if let Some(j) = rand.rand_range(0, i as u64) {
            deck.swap(i, j as usize);
        }
    }
}

// Checks if a move is legal
fn allowed_move( card_chosen: UNOCard, last_card: UNOCard ) -> bool {
    // Not doing Color == NA yet to ensure color choice shenanigans don't occur
    if card_chosen.special == SpecialCard::ColorChange || card_chosen.special == SpecialCard::PlusFour {
        return true; // I'm going to ignore the "you need no playable cards to play draw 4" because screw that rule
    } 
    
    // Special cards on the same special card works
    if card_chosen.special == last_card.special && card_chosen.special != SpecialCard::Base {
        return true;
    }
    
    // Default color-or-number match
    if (card_chosen.special == SpecialCard::Base && card_chosen.number == last_card.number) || card_chosen.color == last_card.color {
        return true;
    }
    
    false // Can't play it!
    
}

// So that if the first drawn card is wild, rnadomly choose a color, but I have to use the random number generator
// So it maps the number to a color
fn color_from_number(num: u8) -> Result<Color> {
    match num {
        0 => Ok(Color::Red),
        1 => Ok(Color::Green),
        2 => Ok(Color::Yellow),
        3 => Ok(Color::Blue),
        4 => Ok(Color::NA),
        _ => Err( Error::new( ErrorKind::Other, "Could not find corresponding color from the given number" ) ),
    }
}

// Builds a new deck and shuffles it
fn refresh_deck(deck:&mut  Vec<UNOCard>, random:&mut Randler) {
    *deck = build_deck();
    shuffle(deck, random);
}

// Checks if there are any plus fours or plus twos in the hand
fn check_countercards(hand: &Vec<UNOCard>) -> bool {
    hand.iter().any(|u: &UNOCard| u.special == SpecialCard::PlusFour || u.special == SpecialCard::PlusTwo)
}

// Ensures the deck is full
// If there is a discard pile, a new deck is made from the discard pile and shuffled
// If there is no discard pile, an entirely new deck is made and shuffled
fn ensure_deck_full(deck: &mut Vec<UNOCard>, discard: &mut Vec<UNOCard>, rand: &mut Randler) {
    if deck.is_empty() {
        if discard.len() > 1 {
            println!("Deck empty. Using discard pile...");
            
            discard.iter_mut().for_each(|c| {
                if c.special == SpecialCard::ColorChange || c.special == SpecialCard::PlusFour {
                    c.color = Color::NA;
                }
            });
            
            let top = discard.pop().unwrap();
            deck.append(discard);
            shuffle(deck, rand);
            discard.push(top);
        } else {
            println!("Deck empty. Using new deck...");
            refresh_deck(deck, rand);
        }
    }
}

// Clears the terminal, but you might just want to enable ANSI escape codes
// If you are on windows, you should probably run the following command in your terminal:
// reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1
fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}

// This is for the AI players
fn get_move_ai(hand: &Vec<UNOCard>, last_played: UNOCard, difficulty: Difficulty, uno: bool) -> Option<usize> {
    
    // To adhere to the +2 stacking force
    if last_played.special == SpecialCard::PlusTwo && check_countercards(hand) {
        if let Some(idx) = hand.iter().position(|c| {
            c.special == SpecialCard::PlusTwo ||
            c.special == SpecialCard::PlusFour
        }) {
            return Some(idx);
        }
    }
    
    match difficulty {
        // Saves special cards for last
        Difficulty::Calm => {
            if let Some(idx) = hand.iter().position(
                |c| { c.special == SpecialCard::Base && allowed_move(*c, last_played) }
            ) {
                return Some(idx);
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                c.special != SpecialCard::Base &&
                c.special != SpecialCard::PlusFour &&
                c.special != SpecialCard::ColorChange &&
                allowed_move(*c, last_played)
            }) {
                return Some(idx);
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                c.special == SpecialCard::PlusFour || c.special == SpecialCard::ColorChange
            }) {
                return Some(idx);
            }
        },
        
        // Uses disruption cards immediately
        Difficulty::Aggressive => {
            if let Some(idx) = hand.iter().position(|c| {
                c.special == SpecialCard::PlusFour
            }) {
                return Some(idx);
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                (c.special == SpecialCard::PlusTwo ||
                c.special == SpecialCard::Skip ||
                c.special == SpecialCard::Reverse) &&
                allowed_move(*c, last_played)
            }) {
                return Some(idx);
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                (c.special == SpecialCard::ColorChange || c.special == SpecialCard::Base) &&
                allowed_move(*c, last_played)
            }) {
                return Some(idx);
            }
        },
        
        // "I lost to this AI twice"
        //                  - Alexandros3015, February 24th, 2026
        // Ts one is impossible without a god hand
        Difficulty::Skilled => {
            if uno {
                if let Some(idx) = hand.iter().position(|c| {
                    c.special != SpecialCard::Base &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
            }
        
        
            let (reds, blues, yellows, greens) = count_color(hand);
            
            if reds > blues && reds > yellows && reds > greens {
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Red &&
                    c.special == SpecialCard::Base &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
                
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Red &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
            }
            else if blues > yellows && blues > greens {
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Blue &&
                    c.special == SpecialCard::Base &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
                
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Blue &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
            }
            else if yellows > greens {
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Yellow &&
                    c.special == SpecialCard::Base &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
                
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Yellow &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
            }
            
            else if greens > 0 {
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Green &&
                    c.special == SpecialCard::Base &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
                
                if let Some(idx) = hand.iter().position(|c| {
                    c.color == Color::Green &&
                    allowed_move(*c, last_played)
                }) {
                    return Some(idx);
                }
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                c.special == SpecialCard::Base &&
                allowed_move(*c, last_played)
            }) {
                return Some(idx);
            }
            
            if let Some(idx) = hand.iter().position(|c| {
                c.special == SpecialCard::ColorChange ||
                c.special == SpecialCard::PlusFour
            }) {
                return Some(idx);
            }
            
            
            
        },
        
    }
    
    // Draw
    None
}

fn count_color(hand: &Vec<UNOCard>) -> (usize, usize, usize, usize) {
    // Counts all colors
    let reds: usize = hand
        .iter()
        .filter(|&card| card.color == Color::Red)
        .count();

    let blues: usize = hand
        .iter()
        .filter(|&card| card.color == Color::Blue)
        .count();
        
    let yellows: usize = hand
        .iter()
        .filter(|&card| card.color == Color::Yellow)
        .count();
        
    let greens: usize = hand
        .iter()
        .filter(|&card| card.color == Color::Green)
        .count();
        
    (reds, blues, yellows, greens)
}

// Gets the most common color on the deck
fn get_common_color(hand: &Vec<UNOCard>, rand: &mut Randler) -> Color {

    let (reds, blues, yellows, greens) = count_color(hand);
    
    // Returns the most common color
    if reds > blues && reds > yellows && reds > greens {
        return Color::Red;
    }
    else if blues > yellows && blues >  greens {
        return Color::Blue;
    } else if yellows > greens {
        return Color::Yellow;
    } else if greens > 0 {
        return Color::Green;
    }
    
    // If there is no common color, return a random color
    color_from_number( rand.rand_range(0, 3).unwrap_or(0) as u8 ).unwrap_or(Color::Red)
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    
    let players: u8 = input("How many players?", "Please enter a proper number that is not too big.");
    let ai_players: u8 = input("How many AI players?", "Please enter a proper number that is not too big.");
    let total_players: u8 = players + ai_players;
    
    let difficulty: Difficulty = input("What AI difficulty? (calm, aggressive, or skilled)", "Please enter a proper difficulty");
    
    let mut rand = Randler::default();
    
    // Warnings
    if total_players == 0 {
        println!("ZERO PLAYERS?? Without a doubt. Right away sir!");
        println!("Player 0 wins? Is this the outcome you desire?");
        return Ok(());
    }
    if total_players == 1 {
        println!("Sure bro, one player");
    } else if total_players == 2 {
        println!("WARNING: Reverse cards now count as skip cards!");
    } else if total_players > 10 {
        println!("WARNING: Playing with this many players may cause unexpected behavior!");
    }
    
    let mut game: Vec<Vec<UNOCard>> = Vec::new(); // All decks
    
    let mut deck = build_deck(); // The deck

    shuffle(&mut deck, &mut rand);
    
    // Give seven cards to each player
    for _ in 0..total_players {
        let mut temp: Vec<UNOCard> = Vec::new();
        for _ in 0..7 {
            if deck.is_empty() {
                println!("Cards expended. Using new deck.");
                refresh_deck(&mut deck, &mut rand);
                        
            }
        
            temp.push( deck.pop().ok_or("Error, out of cards")? );
        }
        game.push(temp);
    }
    
    // Game time:
    
    // The initial card
    let mut last_played: UNOCard = deck.pop().ok_or("Error, out of cards")?; // Promise this'll be the last unsafe thing done with popping
    
    if last_played.color == Color::NA {
        last_played.color = color_from_number( rand.rand_range(0, 3).ok_or("Error with randomization")? as u8 )?;
    }
    
    println!("\n------------\n");
    
    let mut game_state = Game::new(0, total_players as i8,1); // The game state
    let mut add_queue: u32 = 0; // The queue for adding cards to the next player
    let mut getting_added_to: bool; // Whether or not the player is getting cards added to them
    let mut skipped: bool = false; // Whether or not the player has been skipped
    let mut discard: Vec<UNOCard> = Vec::new(); // The discard pile
    let mut uno_detection_panic: bool = false;
    
    loop {
        getting_added_to = true;
        
        let current_idx = game_state.player_number() - 1;

        let player_hand = &mut game[current_idx as usize]; // The player's hand
        
        let is_ai: bool = current_idx >= players as i8;

        player_hand.sort();
        
        println!("\nPlayer #{}'s turn!", game_state.player_number());
        println!("Last card played: {}\n", format_card_message(&last_played));
        
        if is_ai { println!("AI player!"); }
        
        if !is_ai {
        
            for (index, item) in player_hand.iter().enumerate() {
                println!("{}. {}", index + 1,format_card_message(item));
            }
            println!("Type \"d\" or \"draw\" to draw a card");
            println!("Type \"s\" or \"see\" to see the last played card and your hand again");
        }
        
        let countercards = check_countercards(player_hand);
        let mut answer: String;
        let card_selected: Option<UNOCard>;
        loop {
            // If the player cannot counter the current plus two and the adding queue is not empty, then add the cards to the player
            if !countercards && add_queue > 0 {
                card_selected = None;
                getting_added_to = false;
                for _ in 0..add_queue {
                    
                    ensure_deck_full(&mut deck, &mut discard, &mut rand);
                    
                    let drawed: UNOCard = deck.pop().ok_or("Error, out of cards")?;
                    player_hand.push(drawed);
                    println!("Force drawing: {}", format_card_message(&drawed));
                }
                
                add_queue = 0;
                skipped = false;
                break;
            // If the player has been skipped, then skip the card
            } else if skipped {
                println!("You have been skipped!");
                skipped = false;
                card_selected = None;
                break;
            }
            
            player_hand.sort();
            
            if is_ai {
                let ai_move: Option<usize> = get_move_ai(player_hand, last_played, difficulty, uno_detection_panic);
                
                if let Some( play_move ) = ai_move {
                    card_selected = Some(player_hand[play_move]);
                    discard.push( card_selected.unwrap() );
                    player_hand.remove(play_move);
                    
                    println!("AI card selected: {}", format_card_message(&card_selected.unwrap()));
                    break;
                }
                else {
                    ensure_deck_full(&mut deck, &mut discard, &mut rand);
                    
                    let drawed: UNOCard = deck.pop().ok_or("Error, out of cards")?;
                    player_hand.push(drawed);
                    println!("AI drew a card");
                }
            }
            else {
        
    
                println!("What would you like to play (or draw)?");
                answer = input("Enter", "Please enter a card that you have!");
                
                answer = answer.to_lowercase();
                
                // If the player wants to draw a card, then draw a card
                if answer == "draw" || answer == "d" {
                
                    if player_hand.len() == 1 && uno_detection_panic {
                        uno_detection_panic = false;
                    }
                
                    ensure_deck_full(&mut deck, &mut discard, &mut rand);
                    
                    let drawed: UNOCard = deck.pop().ok_or("Error, out of cards")?;
                    player_hand.push(drawed);
                    println!("Drawed card: {}\n", format_card_message(&drawed));
                // Display the last played card and the player's hand
                } else if answer == "s" || answer == "see" {
                    
                    println!("Last card played: {}\n", format_card_message(&last_played));
                    for (index, item) in player_hand.iter().enumerate() {
                        println!("{}. {}", index + 1,format_card_message(item));
                    }
    
                    println!("Type \"d\" or \"draw\" to draw a card");
                    println!("Type \"s\" or \"see\" to see the last played card and your hand again");
                    continue;
                }
                // Parse the answer
                let Ok(answer_usize) = answer.trim().parse::<usize>() else {
                    continue; 
                };
                
                // Ensure the answer is within the bounds of the player's hand
                if answer_usize <= 0 {
                    println!("Please enter a card that you can use");
                    continue;
                }
                
                let answer_usize = (answer_usize -1) as usize; // Zero indexing fix
                
                // Check if the card is valid
                if answer_usize >= player_hand.len() {
                    println!("Please enter a card that you have!\n");
                } else if !allowed_move(player_hand[answer_usize], last_played) {
                    println!("Playing a {} is not allowed. Pick another card or draw.\n", format_card_message(&player_hand[answer_usize]));
                } 
                // If the card is valid, then play it
                else {
                    card_selected = Some(player_hand[answer_usize]);
                    discard.push( card_selected.unwrap() );
                    player_hand.remove(answer_usize);
                    println!("Card selected: {}", format_card_message(&card_selected.unwrap()));
                    break;
                }
            }
        }
        
        // We're gonna do some spins on the rules here 
        // So for one +4s CANNOT be countererd, but they can be played on a +2
        // Adding cards will only work if you have a skip card, if that is the case then you are immune until you play 
        // If not, you're drawing right now
        if let Some(card) = card_selected {
            last_played = card;

            match card.special {
                SpecialCard::PlusFour => {
                
                    if is_ai {
                        last_played.color = get_common_color(player_hand, &mut rand);
                    }
                    else {
                        let chosen_color: Color = input("Enter color", "Please enter an UNO color");
                        last_played.color = chosen_color;
                    }
                    
                    add_queue += 4;
                    getting_added_to = false;
                    skipped = true;
                },
                SpecialCard::PlusTwo => {
                    add_queue += 2;
                    getting_added_to = false;
                },
                SpecialCard::ColorChange => {
                    if is_ai {
                        last_played.color = get_common_color(player_hand, &mut rand);
                    }
                    else {
                        let chosen_color: Color = input("Enter color", "Please enter an UNO color");
                        last_played.color = chosen_color;
                    }
                },
                SpecialCard::Skip => skipped = true,
                SpecialCard::Reverse => {
                    if total_players == 2 {
                        skipped = true;
                    } else {
                        game_state.reverse();
                    }
                },
                SpecialCard::Base => {},
            }
        }
                
        
        // If the player has a countercard but decided not to use it, then they draw at the end of the turn
        if getting_added_to && countercards && add_queue > 0 {
            for _ in 0..add_queue {
                
                ensure_deck_full(&mut deck, &mut discard, &mut rand);
                let drawed: UNOCard = deck.pop().ok_or("Error, out of cards")?;
                player_hand.push(drawed);
                println!("Force drawing: {}", format_card_message(&drawed));
            }
            
            add_queue = 0;
            skipped = false;
        }
        
        // UNO!
        if player_hand.len() == 1 {
            uno_detection_panic = true;
            println!("UNO");
        }
        
        // Exit the loop if a player has won (no cards left)
        if player_hand.len() == 0 {
            println!("Player #{} wins!", game_state.player_number());
            break;
        }
        
        // Clear the terminal and move to the next turn
        let _: String = input("Press enter to continue...", "Error");
        clear_terminal();
        
        
        game_state.next_turn();
    }
    
    // Exit the game
    let _: String = input("Press enter to exit...", "Error");

    Ok(())
}
