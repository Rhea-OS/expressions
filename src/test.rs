#[cfg(test)]
pub mod test {
    use crate::error::*;
    use crate::parse::parse;

    #[test]
    pub fn test() -> Result<()> {
        let result = parse("Hello World")?;

        Ok(())
    }
}