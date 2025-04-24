use std::iter;
use std::rc::Rc;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::*};
use wasm_bindgen::__rt::WasmRefCell;
use expression::error::*;
use expression::Object;
use expression::Value;
use expression::parse;
use expression::parse::literal::Literal;
use crate::DataSource;

#[wasm_bindgen(js_name=Context)]
pub struct Context {
    cx: expression::Context<DataSource>,
    global_context: Rc<WasmRefCell<Object>>
}

/// This function attempts to convert a JS value into its native equivalent.
///
/// * `js_sys::String`s => `expression::Object::String`
/// * `js_sys::Array`s => `expression::Object::List`
/// * `js_sys::Object`s => `expression::Object::AssociativeArray`
/// * `js_sys::Object + { [Symbol.address]: Address }`s => `expression::Object::Address` !
/// * `js_sys::Null` => `expression::Object::Nothing`
/// * `js_sys::falsy()` => `expression::Object::Boolean(false)`
/// * `js_sys::truthy()` => `expression::Object::Boolean(true)`
///
/// > **Note:** Nested values such as those in lists or associative arrays which
/// fail automatic conversation will be dropped silently.
pub(crate) fn js_value_to_object(value: JsValue) -> Option<Object> {
    if let Some(number) = value.as_f64() { 
        return Some(Object::Number(number));
    }
    
    Some(match value {
        value if value.is_null() || value.is_undefined() => Object::Nothing,

        value if value.is_string() => Object::String(value.as_string()?),
        value if value.is_array() => Object::List(js_sys::Array::from(&value)
            .into_iter()
            .flat_map(js_value_to_object)
            .collect()),
        
        value if value.is_function() => {
            let value = value.clone();
            Object::Function(Rc::new(Box::new(move |args| {
                let args = js_sys::Array::from_iter(args.into_iter()
                    .map(value_to_js_object)
                    .map(|i| i.ok_or(Into::<Error>::into(ManualError::ConversionFailed)))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter());
                let result = js_sys::Function::from(value.clone()).call1(&JsValue::null(), &JsValue::from(args))
                    .unwrap_throw();
                js_value_to_object(result)
                    .ok_or(ManualError::ConversionFailed.into())
            })))
        },

        // TODO: Detect Addresses
        value if value.is_object() => expression::Object::AssociativeArray(js_sys::Reflect::own_keys(&value).ok()?
            .into_iter()
            .flat_map(|i| match i {
                value if value.is_string() || value.is_symbol() => value.as_string(),
                _ => None
            })
            .flat_map(|key| js_value_to_object(js_sys::Reflect::get(&value, &JsValue::from_str(&key)).ok()?)
                .map(|value| (key.clone(), value)))
            .collect()),

        value if value.is_falsy() => Object::Boolean(false),
        value if value.is_truthy() => Object::Boolean(true),
        _ => None?,
    })
}


/// This function attempts to convert a native value into its JS equivalent.
///
/// * `js_sys::String`s => `expression::Object::String`
/// * `js_sys::Array`s => `expression::Object::List`
/// * `js_sys::Object`s => `expression::Object::AssociativeArray`
/// * `js_sys::Object + { [Symbol.address]: Address }`s => `expression::Object::Address` !
/// * `js_sys::Null` => `expression::Object::Nothing`
/// * `js_sys::falsy()` => `expression::Object::Boolean(false)`
/// * `js_sys::truthy()` => `expression::Object::Boolean(true)`
///
/// > **Note:** Nested values such as those in lists or associative arrays which
/// fail automatic conversation will be dropped silently.
pub(crate) fn value_to_js_object(value: Object) -> Option<JsValue> {
    Some(match value {
        Object::Number(num) => js_sys::Number::from(num).into(),
        Object::Boolean(bool) => js_sys::Boolean::from(bool).into(),
        Object::String(str) => JsValue::from_str(&str),
        Object::List(list) => JsValue::from(js_sys::Array::from_iter(list.into_iter()
            .flat_map(value_to_js_object))),
        Object::AssociativeArray(arr) => {
            let key_map = js_sys::Object::new();

            for (key, value) in arr {
                js_sys::Reflect::set(&key_map, &JsValue::from_str(&key), &value_to_js_object(value)?)
                    .unwrap_throw();
            }

            JsValue::from(key_map)
        },
        Object::Nothing => JsValue::null(),
        Object::Function(function) => JsClosure::new(move |_args| -> JsValue {
            wasm_bindgen::throw_str("Fuck you");
            function(vec![])
                .map(value_to_js_object)
                .unwrap_throw()
                .unwrap_throw()
        }).into(),
    })
}

#[wasm_bindgen(js_class=Context)]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(config: DataSource) -> Context {
        let cx = config.cx.clone();

        Self {
            global_context: cx.clone(),
            cx: expression::Context::new(config)
                .with_global("cx", Object::function(move |_| {
                    Ok(cx.borrow().clone())
                })),
        }
    }

    #[wasm_bindgen(js_name = pushGlobal)]
    pub fn set_global(&mut self, name: js_sys::JsString, global: JsValue) {
        // Convert the global into the `expression::Object` equivalent

        let Some(name) = name.as_string() else {
            wasm_bindgen::throw_str("Name could not be cast to native string");
        };

        let Some(obj) = js_value_to_object(global) else {
            wasm_bindgen::throw_str("Object could not be cast to target");
        };

        self.cx.push_global(name, obj);
    }

    #[deprecated(note = "Use push_global instead")]
    #[wasm_bindgen(js_name = withGlobal)]
    pub fn with_global(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushGlobal` function instead`")
    }

    #[wasm_bindgen(js_name = pushOperator)]
    pub fn set_operator(&mut self, operator: crate::Operator) {
        self.cx.push_operator(operator.into_operator());
    }

    #[deprecated(note = "Use push_operator instead")]
    #[wasm_bindgen(js_name = withOperator)]
    pub fn with_operator(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushOperator` function instead");
    }

    #[wasm_bindgen(js_name = evaluateStr)]
    pub fn evaluate_str(&self, expression: js_sys::JsString, cx: JsValue) -> JsValue {
        let Some(expr) = expression.as_string() else {
            wasm_bindgen::throw_str("Expression could not be cast to native string");
        };

        *self.global_context.borrow_mut() = js_value_to_object(cx)
            .unwrap_throw();

        let error_message_or_result = self.cx.evaluate(expr)
            .map_err(|error| match error.into_inner() {
                global::Inner::ManualError(ManualError::InsufficientOperands(op)) => format!("The operator '{}' did not receive the required number of operands.", op),
                global::Inner::ManualError(ManualError::CannotCallNonFunctionObject()) => format!("Object not callable"),
                global::Inner::ManualError(ManualError::NoSuchOperator(op)) => format!("The operator '{}' was not recognised", op),
                global::Inner::ManualError(ManualError::NoSuchValue(value)) => format!("'{}' is not defined", value),
                global::Inner::ManualError(ManualError::OperationNotValidForType(op)) => format!("The operation '{}' was attempted on an invalid type", op),
                global::Inner::ManualError(ManualError::EmptyResultSet(query)) => format!("The query '{}' returned no results", query),
                err => format!("Miscellaneous Error: {:?}", err)
            });

        let res = match error_message_or_result {
            Ok(result) => result,
            Err(err) => wasm_bindgen::throw_str(&err)
        };

        if let Some(res) = value_to_js_object(res) {
            res
        } else {
            wasm_bindgen::throw_str("Unable to convert result back into JS");
        }
    }

    #[wasm_bindgen(js_name="parseStr")]
    pub fn parse_str(&self, expr: String) -> Vec<Token> {
        fn flatten(token: &mut Value) -> Option<Vec<Token>> {
            match token {
                Value::Expression(parse::expression::Expression { operator, ref mut operands }) if operands.len() > 0 => {
                    let (first, rest) = operands.split_first_mut()?;

                    Some(flatten(first)?
                        .into_iter()
                        .chain(iter::once(Token::new(operator.clone(), TokenType::Operator)))
                        .chain(rest
                            .into_iter()
                            .filter_map(flatten)
                            .flat_map(|i| i))
                        .collect())
                },
                Value::Expression(parse::expression::Expression { operator, .. }) => Some(vec![Token::new(operator.clone(), TokenType::Operator)]),
                Value::Literal(lit) => Some(vec![match lit {
                    Literal::Name(name) => Token::new(name.clone(), TokenType::Name),
                    Literal::String(str) => Token::new(str.clone(), TokenType::String),
                    Literal::Number(num) => Token::new(format!("{}", num), TokenType::Num),
                    Literal::Address(addr) => Token::new(format!("{{{content}}}", content=addr.query), TokenType::Address),
                }]),
                Value::Call(parse::call::Call { name, arguments }) => {
                    Some(flatten(name)?
                        .into_iter()
                        .chain(iter::once(Token::new("(".to_owned(), TokenType::LParen)))
                        .chain(arguments
                            .iter_mut()
                            .filter_map(flatten)
                            .flat_map(|i| i))
                        .chain(iter::once(Token::new(")".to_owned(), TokenType::RParen)))
                        .collect())
                },
                Value::Access(_) => Some(vec![]),
                Value::List(_) => Some(vec![]),
                Value::AssociativeArray(_) => Some(vec![])
            }
        }

        flatten(&mut match self.cx.parse(&expr) {
            Ok(value) => value,
            Err(err) => wasm_bindgen::throw_str(&format!("{:#?}", err))
        }).unwrap_or(vec![])
    }
}

#[wasm_bindgen]
struct JsClosure {
    closure: Closure<dyn Fn()>,
}

impl JsClosure {
    pub fn new(handler: impl Fn(JsValue) -> JsValue + 'static) -> Self {
        Self {
            closure: Closure::new(|| {})
        }
    }
}


#[wasm_bindgen]
pub struct Token {
    #[wasm_bindgen(js_name="type")]
    pub token_type: TokenType,
    token: String,
//    #[wasm_bindgen(js_name="tokenOffset")]
//    pub token_start: usize,
}

#[wasm_bindgen]
impl Token {
    #[wasm_bindgen]
    pub fn token(&self) -> String {
        self.token.clone()
    }
}

impl Token {
    pub fn new(token: impl AsRef<str>, r#type: TokenType) -> Self {
        Self {
            token_type: r#type,
            token: token.as_ref().to_owned(),
        }
    }
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum TokenType {
    Name,
    Operator,
    LParen,
    LBracket,
    LBrace,
    RParen,
    RBracket,
    RBrace,
    Dot,
    Comma,
    Num,
    String,
    Bool,
    Address
}