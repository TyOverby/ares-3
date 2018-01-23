use super::*;

macro_rules! expect_token_type {
    ($tokens: expr, $expected: pat, $exp_nice: expr) => {
        if $tokens.len() == 0 {
            Err((ParseError::EndOfFileReached, $tokens))
        }
        else {
            match $tokens[0].kind {
                $expected => {Ok((&$tokens[0], &$tokens[1..]))}
                _ => Err((ParseError::UnexpectedToken {
                        found: &$tokens[0],
                        expected: $exp_nice,
                    },
                    &$tokens[1..],
                ))
            }
        }
    };
    ($tokens: expr, $expected_1: pat | $expected_2: pat, $exp_nice: expr) => {
        if $tokens.len() == 0 {
            Err((ParseError::EndOfFileReached, $tokens))
        }
        else {
            match $tokens[0].kind {
                $expected_1 | $expected_2 => {Ok((&$tokens[0], &$tokens[1..]))}
                _ => Err((ParseError::UnexpectedToken {
                        found: &$tokens[0],
                        expected: $exp_nice,
                    },
                    &$tokens[1..],
                ))
            }
        }
    };
}

macro_rules! order_ops {
    (($tokens: expr, $arena: expr, $cache: expr), $final:ident) => {
        $final($tokens, $arena, $cache)
    };
    (($tokens: expr, $arena: expr, $cache: expr), $next:ident, $($prev:ident),+) => {
        $next($tokens, $arena, $cache, &|t, a, c| order_ops!((t, a, c), $($prev),+))
    };
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

pub fn with_cache<'parse, F>(
    cache: &mut ParseCache<'parse>,
    cache_key: CacheKey,
    tokens: &'parse [Token<'parse>],
    func: F,
) -> Result<'parse>
where
    F: FnOnce(&mut ParseCache<'parse>) -> Result<'parse>,
{
    println!(
        "{:?} -> {:?}",
        (tokens.len(), cache_key),
        cache.get(&(tokens.len(), cache_key))
    );
    match cache.get(&(tokens.len(), cache_key)) {
        None => {}
        Some(&CacheState::Working) => return Err((ParseError::Working, tokens)),
        Some(&CacheState::Failed(ref err)) => return Err(err.clone()),
        Some(&CacheState::Done(res)) => return Ok(res),
    }
    cache.insert((tokens.len(), cache_key), CacheState::Working);

    let func_res: Result<'parse> = func(cache);
    match func_res {
        Ok(res) => {
            cache.insert((tokens.len(), cache_key), CacheState::Done(res));
            println!("ok on {:?}", (tokens.len(), cache_key));
            return Ok(res);
        }
        Err(res) => {
            println!("err on {:?}: {:?}", (tokens.len(), cache_key), res);
            cache.insert((tokens.len(), cache_key), CacheState::Failed(res.clone()));
            return Err(res);
        }
    }
}
