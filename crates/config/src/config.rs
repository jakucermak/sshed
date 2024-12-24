use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    general: General,
}

/// Todo: Modify orig
#[derive(Deserialize)]
struct General {
    config_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
