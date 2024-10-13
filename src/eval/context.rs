use crate::{
    error::*,
    eval::Object,
    parse::objects::*,
    parse::parse,
    DataSource,
};
use alloc::{
    vec::Vec,
    borrow::ToOwned,
    boxed::Box,
    string::String,
    string::ToString,
    vec
};
use nom::lib::std::collections::HashMap;
use crate::eval::operators::get_standard_operators;

pub struct Context<Provider: DataSource> {
    globals: HashMap<String, Object>,
    data_provider: Box<Provider>,
    operators: HashMap<String, Operator>,
}

pub(crate) struct Operator {
    handler: Box<dyn Fn(&[Object]) -> Result<Object>>,
    symbol: String,
    precedence: i32,
    operands: usize,
}

pub struct OperatorBuilder {
    handler: Option<Box<dyn Fn(&[Object]) -> Result<Object>>>,
    symbol: Option<String>,
    precedence: i32,
    operands: usize,
}

impl OperatorBuilder {
    pub fn new() -> Self {
        Self {
            handler: None,
            symbol: None,
            precedence: i32::MAX,
            operands: 2,
        }
    }

    pub fn handler(mut self, handler: impl Fn(&[Object]) -> Result<Object> + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    pub fn symbol(mut self, symbol: impl AsRef<str>) -> Self {
        self.symbol = Some(symbol.as_ref().to_owned());
        self
    }

    pub fn precedence(mut self, precedence: i32) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn operands(mut self, operands: usize) -> Self {
        self.operands = operands;
        self
    }

    pub fn build(self) -> Operator {
        if let Some(handler) = self.handler {
            if let Some(symbol) = self.symbol {
                return Operator {
                    handler,
                    symbol,
                    precedence: self.precedence,
                    operands: self.operands,
                };
            }
        }

        panic!("Not all fields defined");
    }
}

impl<Provider> Context<Provider>
where
    Provider: DataSource,
{
    pub fn new(provider: Provider) -> Self {
        Self {
            globals: Default::default(),
            data_provider: Box::new(provider),
            operators: get_standard_operators()
                .into_iter()
                .map(|op| (op.symbol.clone(), op))
                .collect(),
        }
    }

    pub fn with_global(mut self, name: impl AsRef<str>, global: Object) -> Self {
        self.globals.insert(name.as_ref().to_string(), global);
        self
    }

    pub fn with_operator(mut self, operator: Operator) -> Self {
        self.operators.insert(operator.symbol.clone(), operator);
        self
    }

    pub fn resolve_name(&self, name: Key) -> Result<Object> {
        let mut chain = match &name {
            Key::Name(name) => name.split('.').collect::<Vec<_>>(),
            Key::String(name) => vec![name.as_str()],
        }.into_iter();

        'outer: {
            if let Some(obj) = chain.next().and_then(|key| self.globals.get(key)) {
                let mut object = obj;

                while let Some(next) = chain.next() {
                    if let Object::AssociativeArray(arr) = object {
                        if let Some(obj) = arr.get(next) {
                            object = obj;
                        } else {
                            break 'outer;
                        }
                    } else {
                        break 'outer;
                    }
                }

                return Ok(object.clone());
            }
        }

        Err(ManualError::NoSuchValue(match name {
            Key::Name(ref name) | Key::String(ref name) => name.clone()
        }).into())
    }

    pub fn call_object(&self, object: Object, arguments: &[Object]) -> Result<Object> {
        if let Object::Function(obj) = object {
            obj(arguments.iter().cloned().collect())
        } else {
            Err(ManualError::CannotCallNonFunctionObjet().into())
        }
    }

    fn evaluate_value(&self, value: Value) -> Result<Object> {
        match value {
            Value::Expression(Expression { operands, operator }) => if let Some(operator) = self.operators.get(&operator) {
                let operands = operands.into_iter()
                    .map(|operand| self.evaluate_value(operand))
                    .collect::<Result<Vec<_>>>()?;

                Ok((operator.handler)(&operands)?)
            } else {
                Err(ManualError::NoSuchOperator(operator).into())
            }
            Value::Literal(literal) => match literal {
                Literal::Number(number) => Ok(Object::Number(number)),
                Literal::String(string) => Ok(Object::String(string)),
                Literal::Name(name) => self.resolve_name(Key::Name(name.clone())),
                Literal::Address(address) => todo!()
            }
            Value::Call(Call { name, arguments }) => self.call_object(self.resolve_name(name)?, &arguments
                .into_iter()
                .map(|i| self.evaluate_value(i))
                .collect::<Result<Vec<_>>>()?),
            Value::List(_) => todo!(),
            Value::AssociativeArray(_) => todo!()
        }
    }

    pub fn evaluate(&self, expression: impl AsRef<str>) -> Result<Object> {
        Ok(self.evaluate_value(parse(expression.as_ref())?)?)
    }
}