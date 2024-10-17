#[cfg(test)]
mod tests {
    use alloc::vec;
    use crate::{
        eval::context::Context,
        error::*,
        eval::*,
        DataSource,
        Row
    };

    pub struct ManualProvider<Rows: Row> {
        rows: Vec<Rows>,
        columns: Vec<String>,
    }

    impl<Rows: Row> DataSource for ManualProvider<Rows> {
        type Rows = Rows;

        fn list_columns(&self) -> impl Iterator<Item=impl AsRef<str>> {
            self.columns.iter()
        }

        fn rows(&self) -> impl Iterator<Item=&Self::Rows> {
            self.rows.iter()
        }

        fn row_mut(&mut self, row: usize) -> Option<&mut Self::Rows> {
            self.rows.get_mut(row)
        }

        fn num_rows(&self) -> usize {
            self.rows.len()
        }
    }

    struct TwoColumns {
        col1: String,
        col2: String,
    }

    impl Row for TwoColumns {
        fn fields(&self) -> impl Iterator<Item=&str> + Clone {
            vec!["col1", "col2"].into_iter()
        }

        fn get(&self, field: &str) -> Option<Object> {
            match field {
                "col1" => Some(Object::String(self.col1.clone())),
                "col2" => Some(Object::String(self.col2.clone())),
                _ => None
            }
        }
    }

    #[test]
    fn test_eval() -> Result<()> {
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
        });

        assert_eq!(cx.evaluate(r#"1+2"#)?, 3.0);

        Ok(())
    }

    #[test]
    fn test_call() -> Result<()> {
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
        }).with_global("sum", Object::function(|args| operators::add(&args)));

        assert_eq!(cx.evaluate(r#"sum(1,2)"#)?, 3.0);

        Ok(())
    }

    #[test]
    fn test_list() -> Result<()> {
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
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
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
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
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
        }).with_global("list", Object::List(vec![
            Object::Number(1.0),
            Object::Number(2.0),
            Object::Number(3.0),
        ]));

        assert_eq!(cx.evaluate(r#"list.1"#)?, 2.0);

        Ok(())
    }

    #[test]
    fn test_associative_array_access() -> Result<()> {
        let cx = Context::new(ManualProvider::<TwoColumns> {
            columns: vec!["Column 1".to_owned(), "Column 2".to_owned()],
            rows: vec![],
        }).with_global("list", Object::AssociativeArray(vec![
            ("a".to_string(), Object::Number(1.0)),
            ("b".to_string(), Object::Number(2.0)),
            ("c".to_string(), Object::Number(3.0)),
        ].into_iter().collect()));

        assert_eq!(cx.evaluate(r#"list.b"#)?, 2.0);

        Ok(())
    }
}