#![doc = include_str!("../README.md")]
#![no_std]

include! { env!("GENERATED_RS") }

#[cfg(test)]
mod tests {
    use super::*;
    use morse_symbol::Symbol;

    /// Reference morse code for letters and digits, in the same order the
    /// base32 numbering uses: `A..=Z` followed by `0..=9`.
    const LETTERS: &[(u8, &str)] = &[
        (b'A', ".-"),
        (b'B', "-..."),
        (b'C', "-.-."),
        (b'D', "-.."),
        (b'E', "."),
        (b'F', "..-."),
        (b'G', "--."),
        (b'H', "...."),
        (b'I', ".."),
        (b'J', ".---"),
        (b'K', "-.-"),
        (b'L', ".-.."),
        (b'M', "--"),
        (b'N', "-."),
        (b'O', "---"),
        (b'P', ".--."),
        (b'Q', "--.-"),
        (b'R', ".-."),
        (b'S', "..."),
        (b'T', "-"),
        (b'U', "..-"),
        (b'V', "...-"),
        (b'W', ".--"),
        (b'X', "-..-"),
        (b'Y', "-.--"),
        (b'Z', "--.."),
        (b'0', "-----"),
        (b'1', ".----"),
        (b'2', "..---"),
        (b'3', "...--"),
        (b'4', "....-"),
        (b'5', "....."),
        (b'6', "-...."),
        (b'7', "--..."),
        (b'8', "---.."),
        (b'9', "----."),
    ];

    /// Reference morse code for prosigns, with their ascii discriminant
    const PROSIGNS: &[(u8, &str)] = &[
        (1, "-.-.-"),
        (2, "-...-"),
        (4, ".-.-."),
        (6, "...-."),
        (25, "..-.-"),
    ];

    /// Uniform access to the two generated decode state machines
    trait StateMachine: Copy {
        fn start() -> Self;
        fn step(self, symbol: Symbol) -> Self;
        fn finish(self) -> Option<u8>;
    }

    impl StateMachine for ascii::Decode {
        fn start() -> Self {
            Self::new()
        }
        fn step(self, symbol: Symbol) -> Self {
            self.digest(symbol)
        }
        fn finish(self) -> Option<u8> {
            self.finish().map(|v| v as u8)
        }
    }

    impl StateMachine for base32::Decode {
        fn start() -> Self {
            Self::new()
        }
        fn step(self, symbol: Symbol) -> Self {
            self.digest(symbol)
        }
        fn finish(self) -> Option<u8> {
            self.finish().map(|v| v as u8)
        }
    }

    /// Feed a reference code through a state machine and return its byte value
    fn decode<D: StateMachine>(code: &str) -> Option<u8> {
        code.bytes()
            .map(|c| Symbol::from_char(c).unwrap())
            .fold(D::start(), D::step)
            .finish()
    }

    #[test]
    fn ascii_letters_and_digits_finish_to_their_character() {
        for &(expected, code) in LETTERS {
            assert_eq!(
                decode::<ascii::Decode>(code),
                Some(expected),
                "ascii decode of {code:?}"
            );
        }
    }

    #[test]
    fn base32_letters_and_digits_finish_to_their_index() {
        for (index, &(_, code)) in LETTERS.iter().enumerate() {
            // `A..=Z` are 0..=25 and `0..=9` are 26..=35
            assert_eq!(
                decode::<base32::Decode>(code),
                Some(index as u8),
                "base32 decode of {code:?}"
            );
        }
    }

    #[test]
    fn ascii_prosigns_finish_to_their_character() {
        for &(expected, code) in PROSIGNS {
            assert_eq!(
                decode::<ascii::Decode>(code),
                Some(expected),
                "ascii decode of {code:?}"
            );
        }
    }

    #[test]
    fn incomplete_prefix_does_not_finish() {
        // Four lines is the prefix of the digit 0, which has no character by
        // itself
        let prefix = ascii::Decode::new()
            .digest(Symbol::LINE)
            .digest(Symbol::LINE)
            .digest(Symbol::LINE)
            .digest(Symbol::LINE);
        assert_eq!(prefix.finish().map(|v| v as u8), None);

        // The fifth line completes it
        assert_eq!(
            prefix.digest(Symbol::LINE).finish().map(|v| v as u8),
            Some(b'0')
        );
    }

    #[test]
    fn unknown_sequence_restarts_from_the_last_symbol() {
        // Six dots is not a known character, so the state must restart from
        // the single leading symbol
        let mut long = ascii::Decode::new();
        for _ in 0..6 {
            long = long.digest(Symbol::DOT);
        }

        let short = ascii::Decode::new().digest(Symbol::DOT);

        assert_eq!(long as u8, short as u8);
        assert_eq!(long.finish().map(|v| v as u8), Some(b'E'));
    }

    #[test]
    fn group_widening_conversions() {
        use ascii::*;

        assert_eq!(
            LetterOrProsign::from(Letter::LA) as u8,
            LetterOrProsign::LA as u8
        );
        assert_eq!(
            LetterOrProsign::from(Prosign::Attention) as u8,
            LetterOrProsign::Attention as u8
        );
        assert_eq!(
            WordSpaceOrProsign::from(WordSpace::WordSpace) as u8,
            WordSpaceOrProsign::WordSpace as u8
        );
    }

    #[test]
    fn group_narrowing_conversions() {
        use ascii::*;

        assert_eq!(
            Letter::try_from(LetterOrProsign::LA).ok().map(|v| v as u8),
            Some(Letter::LA as u8)
        );
        assert_eq!(
            Letter::try_from(LetterOrProsign::Attention)
                .err()
                .map(|v| v as u8),
            Some(Prosign::Attention as u8)
        );
        assert_eq!(
            WordSpace::try_from(WordSpaceOrProsign::WordSpace)
                .ok()
                .map(|v| v as u8),
            Some(WordSpace::WordSpace as u8)
        );
        assert_eq!(
            WordSpace::try_from(WordSpaceOrProsign::Break)
                .err()
                .map(|v| v as u8),
            Some(Prosign::Break as u8)
        );
    }
}
