use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::number::complete::float;
use nom::sequence::terminated;
use nom::{IResult, Parser};

#[derive(Debug, PartialEq)]
pub enum AmountType {
    Chips(u32),
    Money(f32),
}

impl AmountType {
    pub fn parse(input: &str) -> IResult<&str, AmountType> {
        let amount_chips = map(nom::character::complete::u32, AmountType::Chips);
        let amount_money = map(terminated(float, tag("€")), AmountType::Money);
        let (input, amount) = alt((amount_money, amount_chips)).parse(input)?;
        Ok((input, amount))
    }
}

#[derive(Debug, PartialEq)]
pub enum PokerType {
    HoldemNoLimit,
}

impl PokerType {
    fn parse(input: &str) -> IResult<&str, PokerType> {
        let (input, _) = tag("Holdem no limit").parse(input)?;
        Ok((input, PokerType::HoldemNoLimit))
    }
}
