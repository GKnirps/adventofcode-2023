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

    let winnings = total_winnings(hands.clone());
    println!("your total winnings are {winnings}");

    let winnings_joker = total_winnings_joker(hands);
    println!("your total winnings when playing with a joker are {winnings_joker}");

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

impl Card {
    fn cmp_joker(self, other: Self) -> Ordering {
        if self == Card::J {
            if other == Card::J {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if other == Card::J {
            Ordering::Greater
        } else {
            self.cmp(&other)
        }
    }
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

    fn hand_type_joker(&self) -> HandType {
        let mut counter: [u8; 13] = [0; 13];
        for c in self.0 {
            counter[c as usize] += 1;
        }
        let joker = counter[Card::J as usize];
        counter[Card::J as usize] = 0;
        counter.sort_unstable_by(|l, r| r.cmp(l));
        counter[0] += joker;
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

    fn cmp_joker(&self, other: &Self) -> Ordering {
        self.hand_type_joker()
            .cmp(&other.hand_type_joker())
            .then_with(|| {
                for (l, r) in self.0.iter().zip(other.0.iter()) {
                    let o = l.cmp_joker(*r);
                    if o != Ordering::Equal {
                        return o;
                    }
                }
                Ordering::Equal
            })
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
    hands.sort_by_key(|(hand, _)| *hand);
    hands
        .iter()
        .enumerate()
        .map(|(i, (_, bid))| (i + 1) * *bid as usize)
        .sum()
}

fn total_winnings_joker(mut hands: Vec<(Hand, u32)>) -> usize {
    hands.sort_by(|(l, _), (r, _)| l.cmp_joker(r));
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

    #[test]
    fn total_winnings_joker_works_for_example() {
        // given
        let hands = parse(HANDS).expect("expected successful parsing");

        // when
        let winnings = total_winnings_joker(hands);

        // then
        assert_eq!(winnings, 5905);
    }
}
