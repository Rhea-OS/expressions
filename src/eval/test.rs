#[cfg(test)]
mod tests {
    use crate::error::*;
    use crate::eval::*;

    #[test]
    fn test_eval() -> Result<()> {

        let mut cx = Context::new();

        assert_eq!(cx.evaluate(r#"1+2"#)?, 3.0);

        Ok(())
    }
}