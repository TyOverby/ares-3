#![allow(dead_code)]
use super::*;



pub fn with_cache<'parse, F>(
    cache: &mut ParseCache<'parse>,
    cache_key: CacheKey,
    tokens: &'parse [Token<'parse>],
    func: F,
) -> Result<'parse>
where
    F: FnOnce(&mut ParseCache<'parse>) -> Result<'parse>,
{
    match cache.get(&(tokens.len(), cache_key)) {
        None => {}
        Some(&CacheState::Working) => panic!("infinite parser recursion detected!!!!!"),
        Some(&CacheState::Failed(ref err)) => return Err(err.clone()),
        Some(&CacheState::Done(res)) => return Ok(res),
    }
    cache.insert((tokens.len(), cache_key), CacheState::Working);

    let func_res: Result<'parse> = func(cache);
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
