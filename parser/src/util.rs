use super::*;

macro_rules! expect_token_type {
    ($tokens: expr, $expected: pat, $exp_nice: expr) => {
        match $tokens[0].kind {
            $expected => {Ok((&$tokens[0], &$tokens[1..]))}
            _ => Err((ParseError::UnexpectedToken {
                    found: &$tokens[0],
                    expected: $exp_nice,
                },
                &$tokens[1..],
            ))
        }
    };
}

#[cfg(test)]
macro_rules! matches {
    ($value: expr, $pattern: pat) => {
        match $value {
            $pattern => (),
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

pub fn with_cache<'lex, 'parse, F>(
    cache: &mut ParseCache<'lex, 'parse>,
    cache_key: CacheKey,
    tokens: &'lex [Token<'lex>],
    func: F,
) -> Result<'lex, 'parse>
where
    F: FnOnce(&mut ParseCache<'lex, 'parse>) -> Result<'lex, 'parse>,
{
    match cache.get(&(tokens.len(), cache_key)) {
        None => {}
        Some(&CacheState::Working) => return Err((ParseError::Working, tokens)),
        Some(&CacheState::Failed(ref err)) => return Err(err.clone()),
        Some(&CacheState::Done(res)) => return Ok(res),
    }
    cache.insert((tokens.len(), cache_key), CacheState::Working);

    let func_res: Result<'lex, 'parse> = func(cache);
    match func_res {
        Ok(res) => {
            cache.insert((tokens.len(), cache_key), CacheState::Done(res));
            return Ok(res);
        }
        Err(res) => {
            cache.insert((tokens.len(), cache_key), CacheState::Failed(res.clone()));
            return Err(res);
        }
    }
}
