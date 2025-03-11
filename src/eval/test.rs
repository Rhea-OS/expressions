#[cfg(test)]
mod tests {
    use alloc::vec;
    use core::assert_matches::assert_matches;
    use crate::{
        eval::context::Context,
        error::*,
        eval::*,
        DataSource,
    };

    pub struct ManualProvider {
        values: Vec<String>,
    }

    impl DataSource for ManualProvider {
        fn query(&self, query: impl AsRef<str>) -> Option<Object> {
            let index = query.as_ref().parse::<usize>().ok()?;

            self.values.get(index)
                .map(|i| Object::String(i.clone()))
        }
    }

    #[test]
    fn test_eval() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"1+2"#)?, 3.0);

        Ok(())
    }

    #[test]
    fn test_call() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        }).with_global("sum", Object::function(|args| operators::add(&args)));

        assert_eq!(cx.evaluate(r#"sum(1,2)"#)?, 3.0);

        Ok(())
    }

    #[test]
    fn test_list() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"[1,2,3]"#)?, Object::List(vec![
            Object::Number(1.0),
            Object::Number(2.0),
            Object::Number(3.0)
        ]));

        Ok(())
    }
    
    #[test]
    fn test_associative_array() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"[a=1,b=2,c=3]"#)?, Object::AssociativeArray(vec![
            ("a".to_owned(), Object::Number(1.0)),
            ("b".to_owned(), Object::Number(2.0)),
            ("c".to_owned(), Object::Number(3.0)),
        ].into_iter().collect()));

        Ok(())
    }

    #[test]
    fn test_list_index() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        }).with_global("list", Object::List(vec![
            Object::Number(1.0),
            Object::Number(2.0),
            Object::Number(3.0),
        ]));

        assert_matches!(cx.evaluate(r#"list.1"#), Ok(Object::Number(2.0)));

        Ok(())
    }

    #[test]
    fn test_inline_list_access() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"([1,2]).0"#)?, 1.0);

        Ok(())
    }

    #[test]
    fn test_associative_array_access() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        }).with_global("list", Object::AssociativeArray(vec![
            ("a".to_string(), Object::Number(1.0)),
            ("b".to_string(), Object::Number(2.0)),
            ("c".to_string(), Object::Number(3.0)),
        ].into_iter().collect()));

        assert_eq!(cx.evaluate(r#"list.b"#)?, 2.0);

        Ok(())
    }

    #[test]
    fn test_inline_associative_array_access() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"([x=1,y=2]).x"#)?, 1.0);

        Ok(())
    }

    #[test]
    fn test_data_source() -> Result<()> {
        let cx = Context::new(ManualProvider {
            values: vec!["Hello".to_owned()]
        });

        assert_eq!(cx.evaluate(r#"{0}"#)?, Object::String("Hello".to_owned()));

        Ok(())
    }
}