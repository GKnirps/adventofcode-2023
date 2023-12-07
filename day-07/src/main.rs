use std::cmp::Ordering;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let hands = parse(&content)?;

    let winnings = total_winnings(hands);
    println!("your total points are {winnings}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum HandType {
    High,
    Pair,
    TwoPair,
    Three,
    FullHouse,
    Four,
    Five,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
enum Card {
    Two = 0,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    T,
    J,
    Q,
    K,
    A,
}

fn card_from_byte(c: u8) -> Result<Card, String> {
    match c {
        b'2' => Ok(Card::Two),
        b'3' => Ok(Card::Three),
        b'4' => Ok(Card::Four),
        b'5' => Ok(Card::Five),
        b'6' => Ok(Card::Six),
        b'7' => Ok(Card::Seven),
        b'8' => Ok(Card::Eight),
        b'9' => Ok(Card::Nine),
        b'T' => Ok(Card::T),
        b'J' => Ok(Card::J),
        b'Q' => Ok(Card::Q),
        b'K' => Ok(Card::K),
        b'A' => Ok(Card::A),
        _ => Err(format!("unknown card: '<{c}>'")),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Hand([Card; 5]);

impl Hand {
    fn hand_type(&self) -> HandType {
        let mut counter: [u8; 13] = [0; 13];
        for c in self.0 {
            counter[c as usize] += 1;
        }
        counter.sort_unstable_by(|l, r| r.cmp(l));
        if counter[0] == 5 {
            HandType::Five
        } else if counter[0] == 4 {
            HandType::Four
        } else if counter[0] == 3 && counter[1] == 2 {
            HandType::FullHouse
        } else if counter[0] == 3 {
            HandType::Three
        } else if counter[0] == 2 && counter[1] == 2 {
            HandType::TwoPair
        } else if counter[0] == 2 {
            HandType::Pair
        } else {
            HandType::High
        }
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hand_type()
            .cmp(&other.hand_type())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse(input: &str) -> Result<Vec<(Hand, u32)>, String> {
    input
        .lines()
        .map(|line| {
            let (hand, bid) = line
                .split_once(' ')
                .ok_or_else(|| format!("unable to split line '{line}'"))?;
            let bid = bid
                .parse::<u32>()
                .map_err(|e| format!("unable to parse bid '{bid}': {e}"))?;
            let hand_bytes = hand.as_bytes();
            if hand_bytes.len() != 5 {
                return Err(format!("hand '{hand}' does not have exactly 5 cards"));
            }
            let hand = Hand([
                card_from_byte(hand_bytes[0])?,
                card_from_byte(hand_bytes[1])?,
                card_from_byte(hand_bytes[2])?,
                card_from_byte(hand_bytes[3])?,
                card_from_byte(hand_bytes[4])?,
            ]);

            Ok((hand, bid))
        })
        .collect()
}

fn total_winnings(mut hands: Vec<(Hand, u32)>) -> usize {
    hands.sort_unstable_by_key(|(hand, _)| *hand);
    hands
        .iter()
        .enumerate()
        .map(|(i, (_, bid))| (i + 1) * *bid as usize)
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const HANDS: &str = r#"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483
"#;

    #[test]
    fn total_winnings_works_for_example() {
        // given
        let hands = parse(HANDS).expect("expected successful parsing");

        // when
        let winnings = total_winnings(hands);

        // then
        assert_eq!(winnings, 6440);
    }
}
