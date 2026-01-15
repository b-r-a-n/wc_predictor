//! Knockout stage types.

use serde::{Deserialize, Serialize};

use crate::match_result::MatchResult;

/// Knockout round identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnockoutRound {
    /// Round of 32 (first knockout round, 16 matches)
    RoundOf32,
    /// Round of 16 (8 matches)
    RoundOf16,
    /// Quarter-finals (4 matches)
    QuarterFinal,
    /// Semi-finals (2 matches)
    SemiFinal,
    /// Third-place playoff
    ThirdPlace,
    /// Final
    Final,
}

impl KnockoutRound {
    /// Get the number of matches in this round.
    pub fn num_matches(&self) -> usize {
        match self {
            Self::RoundOf32 => 16,
            Self::RoundOf16 => 8,
            Self::QuarterFinal => 4,
            Self::SemiFinal => 2,
            Self::ThirdPlace => 1,
            Self::Final => 1,
        }
    }

    /// Get the next round (if any).
    pub fn next_round(&self) -> Option<Self> {
        match self {
            Self::RoundOf32 => Some(Self::RoundOf16),
            Self::RoundOf16 => Some(Self::QuarterFinal),
            Self::QuarterFinal => Some(Self::SemiFinal),
            Self::SemiFinal => None, // Splits into Final and ThirdPlace
            Self::ThirdPlace => None,
            Self::Final => None,
        }
    }

    /// Get a human-readable name for this round.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RoundOf32 => "Round of 32",
            Self::RoundOf16 => "Round of 16",
            Self::QuarterFinal => "Quarter-finals",
            Self::SemiFinal => "Semi-finals",
            Self::ThirdPlace => "Third-place match",
            Self::Final => "Final",
        }
    }

    /// Get the importance factor for this round (used in prediction).
    /// Higher values indicate more important matches.
    pub fn importance(&self) -> f64 {
        match self {
            Self::RoundOf32 => 1.5,
            Self::RoundOf16 => 2.0,
            Self::QuarterFinal => 2.5,
            Self::SemiFinal => 3.0,
            Self::ThirdPlace => 2.0,
            Self::Final => 4.0,
        }
    }
}

/// Complete knockout bracket with all results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnockoutBracket {
    /// Round of 32 results (16 matches)
    pub round_of_32: Vec<MatchResult>,
    /// Round of 16 results (8 matches)
    pub round_of_16: Vec<MatchResult>,
    /// Quarter-final results (4 matches)
    pub quarter_finals: Vec<MatchResult>,
    /// Semi-final results (2 matches)
    pub semi_finals: Vec<MatchResult>,
    /// Third-place match result
    pub third_place: MatchResult,
    /// Final match result
    pub final_match: MatchResult,
}

impl KnockoutBracket {
    /// Get all matches in a specific round.
    pub fn get_round(&self, round: KnockoutRound) -> &[MatchResult] {
        match round {
            KnockoutRound::RoundOf32 => &self.round_of_32,
            KnockoutRound::RoundOf16 => &self.round_of_16,
            KnockoutRound::QuarterFinal => &self.quarter_finals,
            KnockoutRound::SemiFinal => &self.semi_finals,
            KnockoutRound::ThirdPlace => std::slice::from_ref(&self.third_place),
            KnockoutRound::Final => std::slice::from_ref(&self.final_match),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_num_matches() {
        assert_eq!(KnockoutRound::RoundOf32.num_matches(), 16);
        assert_eq!(KnockoutRound::RoundOf16.num_matches(), 8);
        assert_eq!(KnockoutRound::QuarterFinal.num_matches(), 4);
        assert_eq!(KnockoutRound::SemiFinal.num_matches(), 2);
        assert_eq!(KnockoutRound::Final.num_matches(), 1);
    }

    #[test]
    fn test_round_progression() {
        assert_eq!(
            KnockoutRound::RoundOf32.next_round(),
            Some(KnockoutRound::RoundOf16)
        );
        assert_eq!(
            KnockoutRound::RoundOf16.next_round(),
            Some(KnockoutRound::QuarterFinal)
        );
        assert_eq!(KnockoutRound::Final.next_round(), None);
    }
}
