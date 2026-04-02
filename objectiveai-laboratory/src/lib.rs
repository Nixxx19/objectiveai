pub fn hello() -> &'static str {
    "objectiveai-laboratory"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(hello(), "objectiveai-laboratory");
    }
}
