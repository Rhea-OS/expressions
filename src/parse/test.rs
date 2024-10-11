#[cfg(test)]
pub mod test {
    use crate::error::*;
    use crate::parse::parse;

    use crate::parse::*;
    use crate::parse::expression::Expression;
    use crate::parse::literal::Literal;

    #[test]
    pub fn test_extremely_simple_expression() -> Result<()> {
        assert_eq!(parse("1+2")?, Value::Expression(Expression {
            operands: vec![
                Value::Literal(Literal::Number(1.0)),
                Value::Literal(Literal::Number(2.0)),
            ],
            operator: "+"
        }));

        Ok(())
    }

    #[test]
    pub fn test_oder_of_operations() -> Result<()> {
        assert_eq!(parse("1*2+3^2")?, Value::Expression(Expression {
            operands: vec![
                Value::Expression(Expression {
                    operands: vec![
                        Value::Literal(Literal::Number(1.0)),
                        Value::Literal(Literal::Number(2.0)),
                    ],
                    operator: "*"
                }),
                Value::Expression(Expression {
                    operands: vec![
                        Value::Literal(Literal::Number(3.0)),
                        Value::Literal(Literal::Number(2.0)),
                    ],
                    operator: "^"
                })
            ],
            operator: "+"
        }));

        Ok(())
    }

    #[test]
    pub fn test_call() -> Result<()> {
        assert_eq!(parse("hello()")?, Value::Expression(Expression {
            operands: vec![
                Value::Expression(Expression {
                    operands: vec![
                        Value::Literal(Literal::Number(1.0)),
                        Value::Literal(Literal::Number(2.0)),
                    ],
                    operator: "*"
                }),
                Value::Literal(Literal::Number(3.0))
            ],
            operator: "+"
        }));

        Ok(())
    }
}