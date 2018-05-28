macro_rules! precedence {
    ($last: ident) => {
        &|t, a| $last(t, a)
    };
    ($current: ident, $($rest: ident),+) => {{
         &|t, a| $current(t, a, precedence!($($rest),+))
    }};
}

#[cfg(test)]
macro_rules! matches {
    ($value: expr, $pattern: pat) => {
        match $value {
            $pattern => true,
            _ => panic!("match failed, actual: {:#?}", $value),
        }
    };
    ($value: expr, $pattern: pat, $($exp: expr),*) => {
        match $value {
            $pattern => {$(assert!($exp));* ; true},
            _ => panic!("match failed, actual: {:#?}", $value),
        }
    };
}

macro_rules! expect_token_type {
    ($tokens:expr, $expected:pat, $exp_nice:expr) => {
        if $tokens.len() == 0 {
            Err((ParseError::EndOfFileReached, $tokens))
        } else {
            match $tokens[0].kind {
                $expected => Ok((&$tokens[0], &$tokens[1..])),
                _ => Err((
                    ParseError::UnexpectedToken {
                        found: &$tokens[0],
                        expected: $exp_nice,
                    },
                    &$tokens[1..],
                )),
            }
        }
    };
    ($tokens:expr, $expected_1:pat | $expected_2:pat, $exp_nice:expr) => {
        if $tokens.len() == 0 {
            Err((ParseError::EndOfFileReached, $tokens))
        } else {
            match $tokens[0].kind {
                $expected_1 | $expected_2 => Ok((&$tokens[0], &$tokens[1..])),
                _ => Err((
                    ParseError::UnexpectedToken {
                        found: &$tokens[0],
                        expected: $exp_nice,
                    },
                    &$tokens[1..],
                )),
            }
        }
    };
}

macro_rules! me_or_fallback {
    ($me:ident, $lower:ident,($tokens:expr, $arena:expr)) => {
        $me($tokens, $arena, $lower).or_else(|_| $lower($tokens, $arena))
    };
}
