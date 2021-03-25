use unicase::UniCase;

pub fn phf_str_lookup<'k>(map: &phf::Map<&'static str, usize>, key: &str) -> Option<usize> {
    // safety: It is safe to temporarily treat the key as 'static within this function,
    // the map is immutable, and cannot store a reference to this key.
    let key: &'static str = unsafe { std::mem::transmute(key) };
    map.get(&key).map(|index| *index)
}

pub fn phf_unicase_lookup<'k>(
    map: &phf::Map<UniCase<&'static str>, usize>,
    key: &str,
) -> Option<usize> {
    // safety: It is safe to temporarily treat the key as 'static within this function,
    // the map is immutable, and cannot store a reference to this key.
    let key: UniCase<&'static str> = unsafe { std::mem::transmute(UniCase::new(key)) };
    map.get(&key).map(|index| *index)
}
