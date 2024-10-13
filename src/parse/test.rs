#[cfg(test)]
pub mod test {
    use core::assert_matches::assert_matches;
    use crate::error::*;

    use crate::parse::call::Call;
    use crate::parse::expression::Expression;
    use crate::parse::key::Key;
    use crate::parse::literal::Literal;
    use crate::parse::*;
    use crate::parse::associative_array::AssociativeArray;
    use crate::parse::list::List;

    /// Operators is a static map of ("Token", "Precedence", "NumOperands")
    static OPERATORS: &'static [(&'static str, i64, u64)] = &[
        ("==", 1, 2),
        ("!=", 1, 2),
        ("&&", 3, 2),
        ("||", 3, 2),
        ("!", 3, 1),
        (">", 5, 2),
        ("<", 5, 2),
        ("+", 10, 2),
        ("-", 10, 2),
        ("*", 15, 2),
        ("/", 15, 2),
        ("%", 15, 2),
        ("^", 20, 2),
    ];

    fn parse(input: impl AsRef<str>) -> Result<Value> {
        // Group the operators by precedence into a BTreeMap so it's sorted.
        let operators = OPERATORS.iter()
            .fold(BTreeMap::new(), |mut accumulator, (token, precedence, _num_operands)| {
                if !accumulator.contains_key(precedence) {
                    accumulator.insert(*precedence, vec![]);
                }

                accumulator.get_mut(precedence).unwrap().push(*token);

                return accumulator;
            });

        let parser = value_parser(ParseContext::new(
            operators.keys().copied().collect::<Vec<_>>(),
            operators.into_iter()
                .map(|(precedence, tokens)| (precedence, tokens.into_iter().map(|i| i.to_owned()).collect::<Vec<_>>()))
                .collect(),
        ));

        parser(input.as_ref())
            .map(|(_, v)| v)
            .map_err(|err| stringify(err).into())
    }

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
    pub fn test_names() -> Result<()> {
        assert_eq!(parse("a")?, Value::Literal(Literal::Name("a".to_owned())));
        assert_matches!(Key::parse("0"), Err(_));
        assert_matches!(Key::parse("."), Err(_));
        assert_matches!(Key::parse(":"), Err(_));
        assert_matches!(Key::parse("a.0"), Ok(("", Key::Name(name))) if name == "a.0");

        Ok(())
    }

    #[test]
    pub fn test_string() -> Result<()> {
        assert_matches!(Key::parse(r#""Hello World""#), Ok(("", Key::String(name))) if name == "Hello World");
        assert_matches!(Key::parse(r#""Hello\nWorld""#), Ok(("", Key::String(name))) if name == "Hello\nWorld");
        assert_matches!(Key::parse(r#""Hello\"World""#), Ok(("", Key::String(name))) if name == "Hello\"World");
        assert_matches!(Key::parse(r#""Hello"World""#), Ok(("World\"", Key::String(name))) if name == "Hello");

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