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

        fn tuples(&self) -> impl Iterator<Item=&Self::Rows> {
            self.rows.iter()
        }

        fn tuples_mut(&mut self) -> impl Iterator<Item=&mut Self::Rows> {
            self.rows.iter_mut()
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
}