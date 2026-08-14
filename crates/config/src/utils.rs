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

// Error wrapper
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDe(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSer(e)
    }
}
