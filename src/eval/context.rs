use crate::error::*;
use crate::eval::globals::get_standard_globals;
use crate::eval::operators::get_standard_operators;
use crate::eval::Object;
use crate::parse::objects::*;
use crate::DataSource;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
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
/// use expression::Context;
/// use expression::EmptyProvider;
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

#[derive(Clone)]
pub struct Operator {
    handler: Rc<Box<dyn Fn(&[Object]) -> Result<Object>>>,
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
                    handler: Rc::new(handler),
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
            globals: get_standard_globals().into_iter().collect(),
            data_provider: Box::new(provider),
            operators: get_standard_operators().into_iter().map(|op| (op.symbol.clone(), op)).collect(),
        }
    }

    /// # Globals
    /// The `Context` is where functions, variables and other values are registered.
    /// These may allow interaction with the host system, mathematical functions or other useful utility functions.
    ///
    /// ```rust
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let cx =
    ///     Context::new(EmptyProvider::new()).with_global("PI", Object::Number(std::f64::consts::PI));
    ///
    /// assert_eq!(cx.evaluate(r#"PI"#).unwrap(), std::f64::consts::PI);
    /// ```
    pub fn with_global(mut self, name: impl AsRef<str>, global: Object) -> Self {
        self.globals.insert(name.as_ref().to_string(), global);
        self
    }

    /// # Globals
    /// The `Context` is where functions, variables and other values are registered.
    /// These may allow interaction with the host system, mathematical functions or other useful utility functions.
    ///
    /// ```rust
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let mut cx = Context::new(EmptyProvider::new());
    /// cx.push_global("PI", Object::Number(std::f64::consts::PI));
    ///
    /// assert_eq!(cx.evaluate(r#"PI"#).unwrap(), std::f64::consts::PI);
    /// ```
    pub fn push_global(&mut self, name: impl AsRef<str>, global: Object) {
        self.globals.insert(name.as_ref().to_string(), global);
    }

    /// # Operator Overloads
    /// Operators are defined on the context object. These can be overridden, to produce custom operator behaviour.
    ///
    /// ```rust
    /// use expression::eval::context::OperatorBuilder;
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let cx = Context::new(EmptyProvider::new()).with_operator(
    ///     OperatorBuilder::new()
    ///         .symbol("~")
    ///         .operands(1)
    ///         .precedence(10)
    ///         .handler(|args| {
    ///             // Define an operator which nullifies the value
    ///             Ok(Object::Nothing)
    ///         })
    ///         .build(),
    /// );
    ///
    /// assert_eq!(cx.evaluate(r#"2~10"#).unwrap(), Object::Nothing);
    /// ```
    pub fn with_operator(mut self, operator: Operator) -> Self {
        self.operators.insert(operator.symbol.clone(), operator);
        self
    }

    /// # Operator Overloads
    /// Operators are defined on the context object. These can be overridden, to produce custom operator behaviour.
    ///
    /// ```rust
    /// use expression::eval::context::OperatorBuilder;
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let mut cx = Context::new(EmptyProvider::new());
    /// cx.push_operator(
    ///     OperatorBuilder::new()
    ///         .symbol("~")
    ///         .operands(1)
    ///         .precedence(10)
    ///         .handler(|args| {
    ///             // Define an operator which nullifies the value
    ///             Ok(Object::Nothing)
    ///         })
    ///         .build(),
    /// );
    ///
    /// assert_eq!(cx.evaluate(r#"2~10"#).unwrap(), Object::Nothing);
    /// ```
    pub fn push_operator(&mut self, operator: Operator) {
        self.operators.insert(operator.symbol.clone(), operator);
    }

    pub fn call_object(&self, object: Object, arguments: &[Object]) -> Result<Object> {
        if let Object::Function(obj) = object {
            obj(arguments.iter().cloned().collect())
        } else {
            Err(ManualError::CannotCallNonFunctionObject().into())
        }
    }

    fn query(&self, query: impl AsRef<str>) -> Option<Object> {
        Some(self.data_provider.query(query)?)
    }

    fn evaluate_value(&self, value: Value) -> Result<Object> {
        match value {
            Value::Expression(Expression { operands, operator }) =>
                if let Some(operator) = self.operators.get(&operator) {
                    let operands = operands
                        .into_iter()
                        .map(|operand| self.evaluate_value(operand))
                        .collect::<Result<Vec<_>>>()?;

                    Ok((operator.handler)(&operands)?)
                } else {
                    Err(ManualError::NoSuchOperator(operator).into())
                },

            Value::Literal(literal) => match literal {
                Literal::Nothing => Ok(Object::Nothing),
                Literal::Bool(bool) => Ok(Object::Boolean(bool)),
                Literal::Number(number) => Ok(Object::Number(number)),
                Literal::String(string) => Ok(Object::String(string)),
                Literal::Name(name) => self
                    .globals
                    .get(name.as_str())
                    .cloned()
                    .ok_or(ManualError::NoSuchValue(name.clone()).into()),

                Literal::Address(address) => Ok(self.query(&address.query).unwrap_or(Object::Nothing)),
            },

            Value::Call(Call { name, arguments }) => self.call_object(
                self.evaluate_value(*name)?,
                &arguments
                    .into_iter()
                    .map(|i| self.evaluate_value(i))
                    .collect::<Result<Vec<_>>>()?,
            ),

            Value::Access(Access { left, member }) => match (self.evaluate_value(*left), member) {
                (Ok(Object::AssociativeArray(array)), Literal::Name(ref name) | Literal::String(ref name)) => array.get(name).cloned().ok_or(ManualError::NoSuchValue(name.clone()).into()),

                (Ok(Object::List(list)), Literal::Name(ref name) | Literal::String(ref name)) => name
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| list.get(index))
                    .cloned()
                    .ok_or(ManualError::NoSuchValue(name.clone()).into()),

                (Ok(Object::List(list)), Literal::Number(ref name)) => list
                    .get(*name as usize)
                    .cloned()
                    .ok_or(ManualError::NoSuchValue(format!("{}", name)).into()),

                (Ok(obj), _) => Err(ManualError::OperationNotValidForType(format!("Object of type '{}' does not exhibit any accessible members", obj.datatype())).into()),
                (Err(err), _) => Err(err),
            },

            Value::List(list) => Ok(Object::List(
                list.items
                    .into_iter()
                    .map(|i| self.evaluate_value(i))
                    .collect::<Result<Vec<_>>>()?,
            )),

            Value::AssociativeArray(arr) => Ok(Object::AssociativeArray(
                arr.items
                    .into_iter()
                    .map(|(key, value)| {
                        self.evaluate_value(value).map(|value| {
                            (
                                match key {
                                    Key::Name(str) | Key::String(str) => str,
                                },
                                value,
                            )
                        })
                    })
                    .collect::<Result<HashMap<_, _>>>()?,
            )),
        }
    }

    pub fn evaluate(&self, expression: impl AsRef<str>) -> Result<Object> {
        let ast = self.parse(expression.as_ref())?;
        Ok(self.evaluate_value(ast)?)
    }

    pub fn provider(&self) -> &Provider {
        self.data_provider.as_ref()
    }

    pub fn provider_mut(&mut self) -> &mut Provider {
        self.data_provider.as_mut()
    }
}

impl<Provider: DataSource + Clone + 'static> Context<Provider> {
    /// # Functions
    /// Registers a function on the context. This function is a utility function that makes reusing the context object more convenient.
    ///
    /// ```rust
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let mut cx = Context::new(EmptyProvider::new());
    /// cx.push_fn("example", |cx, args| {
    ///     println!("{:?}", &cx.provider());
    ///
    ///     Ok(Object::Nothing)
    /// });
    ///
    /// // Prints the provider.
    /// assert_eq!(cx.evaluate(r#"example()"#).unwrap(), Object::Nothing);
    /// ```
    pub fn push_fn<F>(&mut self, name: impl AsRef<str>, f: F) -> &mut Self
    where
        F: Fn(Self, &[Object]) -> Result<Object> + 'static, {
        let cx = self.clone();
        self.push_global(name, Object::function(move |args| f(cx.clone(), &args)));

        self
    }

    /// # Functions
    /// Registers a function on the context. This function is a utility function that makes reusing the context object more convenient.
    ///
    /// ```rust
    /// use expression::eval::Object;
    /// use expression::Context;
    /// use expression::DataSource;
    /// use expression::EmptyProvider;
    ///
    /// let mut cx = Context::new(EmptyProvider::new()).with_fn("example", |cx, args| {
    ///     println!("{:?}", &cx.provider());
    ///
    ///     Ok(Object::Nothing)
    /// });
    ///
    /// // Prints the provider.
    /// assert_eq!(cx.evaluate(r#"example()"#).unwrap(), Object::Nothing);
    /// ```
    pub fn with_fn<F>(mut self, name: impl AsRef<str>, f: F) -> Self
    where
        F: Fn(Self, &[Object]) -> Result<Object> + 'static, {
        self.push_fn(name, f);
        self
    }
}

impl<Provider: DataSource + Clone> Clone for Context<Provider> {
    fn clone(&self) -> Self {
        Self {
            globals: self.globals.clone(),
            data_provider: self.data_provider.clone(),
            operators: self.operators.clone(),
        }
    }
}
