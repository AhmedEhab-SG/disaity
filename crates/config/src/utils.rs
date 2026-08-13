use toml::Value;

pub trait Merge {
    fn safe_merge(base: &mut Value, over: Value) {
        match (base, over) {
            (Value::Table(b), Value::Table(o)) => {
                for (k, v) in o {
                    match b.get_mut(&k) {
                        Some(bv) => Self::safe_merge(bv, v),
                        None => {
                            b.insert(k, v);
                        }
                    }
                }
            }

            (b, o) => *b = o,
        }
    }
}
