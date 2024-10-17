use crate::{
    eval::operators::get_standard_operators,
    error::*,
    eval::Object,
    parse::objects::*,
    DataSource
};
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::String,
    string::ToString,
    vec,
    vec::Vec,
};
use nom::lib::std::collections::HashMap;

/// # Context
///
/// Stores and holds data relevant to expression parsing and evaluation.
///
/// Contexts depend on a data source which provides data for use within expressions.
/// Throughout these docs, the `EmptyProvider` is used as a placeholder.
/// In real life, you'll need to [implement your own](DataSource).
///
/// ## Example
/// ```rust
/// use expression::{Context, EmptyProvider};
///
/// let cx = Context::new(EmptyProvider::new());
///
/// assert_eq!(cx.evaluate(r#"2*5"#).unwrap(), 10.0);
/// ```
pub struct Context<Provider: DataSource> {
    globals: HashMap<String, Object>,
    data_provider: Box<Provider>,
    pub(crate) operators: HashMap<String, Operator>,
}

pub struct Operator {
    handler: Box<dyn Fn(&[Object]) -> Result<Object>>,
    symbol: String,
    pub(crate) precedence: i64,
    operands: usize,
}

pub struct OperatorBuilder {
    handler: Option<Box<dyn Fn(&[Object]) -> Result<Object>>>,
    symbol: Option<String>,
    precedence: i64,
    operands: usize,
}

impl OperatorBuilder {
    pub fn new() -> Self {
        Self {
            handler: None,
            symbol: None,
            precedence: i64::MAX,
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

    pub fn precedence(mut self, precedence: i64) -> Self {
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

    /// # Globals
    /// The `Context` is where functions, variables and other values are registered.
    /// These may allow interaction with the host system, mathematical functions or other useful utility functions.
    ///
    /// ```rust
    /// use expression::{
    ///     Context,
    ///     DataSource,
    ///     EmptyProvider,
    ///     Row,
    ///     eval::Object
    /// };
    ///
    /// let cx = Context::new(EmptyProvider::new()).with_global("PI", Object::Number(std::f64::consts::PI));
    ///
    /// assert_eq!(cx.evaluate(r#"PI"#).unwrap(), std::f64::consts::PI);
    /// ```
    pub fn with_global(mut self, name: impl AsRef<str>, global: Object) -> Self {
        self.globals.insert(name.as_ref().to_string(), global);
        self
    }

    /// # Operator Overloads
    /// Operators are defined on the context object. These can be overridden, to produce custom operator behaviour.
    ///
    /// ```rust
    /// use expression::{
    ///     Context,
    ///     DataSource,
    ///     EmptyProvider,
    ///     Row,
    ///     eval::Object,
    ///     eval::context::OperatorBuilder
    /// };
    ///
    /// let cx = Context::new(EmptyProvider::new())
    ///     .with_operator(OperatorBuilder::new()
    ///         .symbol("~")
    ///         .operands(1)
    ///         .precedence(10)
    ///         .handler(|args| {
    ///
    ///             // Define an operator which nullifies the value
    ///             Ok(Object::Nothing)
    ///         })
    ///         .build());
    ///
    /// assert_eq!(cx.evaluate(r#"~10"#).unwrap(), Object::Nothing);
    /// ```
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
                    match object {
                        Object::AssociativeArray(arr) => if let Some(obj) = arr.get(next) {
                            object = obj;
                        } else {
                            break 'outer;
                        },
                        Object::List(list) => if let Ok(Some(obj)) = next.parse::<usize>().map(|a| list.get(a)) {
                            object = obj
                        } else {
                            break 'outer;
                        },
                        _ => break 'outer,
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

            Value::List(list) => Ok(Object::List(list.items.into_iter()
                .map(|i| self.evaluate_value(i))
                .collect::<Result<Vec<_>>>()?)),

            Value::AssociativeArray(arr) => Ok(Object::AssociativeArray(arr.items.into_iter()
                .map(|(key, value)| self.evaluate_value(value)
                    .map(|value| (match key {
                        Key::Name(str) | Key::String(str) => str
                    }, value)))
                .collect::<Result<HashMap<_, _>>>()?))
        }
    }

    pub fn evaluate(&self, expression: impl AsRef<str>) -> Result<Object> {
        Ok(self.evaluate_value(self.parse(expression.as_ref())?)?)
    }
}