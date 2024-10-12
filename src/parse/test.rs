#[cfg(test)]
pub mod test {
    use crate::error::*;
    use crate::parse::parse;

    use crate::parse::call::Call;
    use crate::parse::expression::Expression;
    use crate::parse::key::Key;
    use crate::parse::literal::Literal;
    use crate::parse::*;
    use crate::parse::associative_array::AssociativeArray;
    use crate::parse::list::List;

    #[test]
    pub fn test_extremely_simple_expression() -> Result<()> {
        let expr = parse("1+2");

        assert_eq!(expr?, Value::Expression(Expression {
            operands: vec![
                Value::Literal(Literal::Number(1.0)),
                Value::Literal(Literal::Number(2.0)),
            ],
            operator: "+".to_owned()
        }));

        Ok(())
    }

    #[test]
    pub fn test_oder_of_operations() -> Result<()> {
        let expr = parse("1*2+3^2");

        assert_eq!(expr?, Value::Expression(Expression {
            operands: vec![
                Value::Expression(Expression {
                    operands: vec![
                        Value::Literal(Literal::Number(1.0)),
                        Value::Literal(Literal::Number(2.0)),
                    ],
                    operator: "*".to_owned()
                }),
                Value::Expression(Expression {
                    operands: vec![
                        Value::Literal(Literal::Number(3.0)),
                        Value::Literal(Literal::Number(2.0)),
                    ],
                    operator: "^".to_owned()
                })
            ],
            operator: "+".to_owned()
        }));

        Ok(())
    }

    #[test]
    pub fn test_parentheses() -> Result<()> {
        let expr = parse("1*(2+3)^4");

        assert_eq!(expr?, Value::Expression(Expression {
            operands: vec![
                Value::Literal(Literal::Number(1.0)),
                Value::Expression(Expression {
                    operands: vec![
                        Value::Expression(Expression {
                            operands: vec![
                                Value::Literal(Literal::Number(2.0)),
                                Value::Literal(Literal::Number(3.0)),
                            ],
                            operator: "+".to_owned()
                        }),
                        Value::Literal(Literal::Number(4.0)),
                    ],
                    operator: "^".to_owned()
                })
            ],
            operator: "*".to_owned()
        }));

        Ok(())
    }

    #[test]
    pub fn test_call() -> Result<()> {
        assert_eq!(parse("hello(1)")?, Value::Call(Call {
            name: Key::Name("hello".to_owned()),
            arguments: vec![Value::Literal(Literal::Number(1.0))],
        }));

        Ok(())
    }

    #[test]
    pub fn test_list() -> Result<()> {
        assert_eq!(parse("[1,2,3]")?, Value::List(List {
            items: vec![
                Value::Literal(Literal::Number(1.0)),
                Value::Literal(Literal::Number(2.0)),
                Value::Literal(Literal::Number(3.0)),
            ]
        }));

        Ok(())
    }

    #[test]
    pub fn test_associative_array() -> Result<()> {
        assert_eq!(parse("[tomato=1,beans=2,cheese=3]")?, Value::AssociativeArray(AssociativeArray {
            items: vec![
                (Key::Name("tomato".into()), Value::Literal(Literal::Number(1.0))),
                (Key::Name("beans".into()), Value::Literal(Literal::Number(2.0))),
                (Key::Name("cheese".into()), Value::Literal(Literal::Number(3.0))),
            ]
        }));

        Ok(())
    }
}