use std::iter;
use std::rc::Rc;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::*};
use wasm_bindgen::__rt::WasmRefCell;
use expression::error::*;
use expression::Object;
use expression::Value;
use expression::parse;
use expression::parse::literal::Literal;
use crate::convert::{js_value_to_object, value_to_js_object};
use crate::DataSource;

#[wasm_bindgen(js_name=Context)]
pub struct Context {
    expr: expression::Context<DataSource>,
    global_context: Rc<WasmRefCell<Object>>
}

#[wasm_bindgen(js_class=Context)]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new(provider: js_sys::Object) -> Context {
        let provider = DataSource::from_js(provider);

        Self {
            global_context: provider.ephemeral_cx.clone(),
            expr: expression::Context::new(provider)
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

        self.expr.push_global(name, obj);
    }

    #[deprecated(note = "Use pushGlobal instead")]
    #[wasm_bindgen(js_name = withGlobal)]
    pub fn with_global(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushGlobal` function instead`")
    }

    #[wasm_bindgen(js_name = pushOperator)]
    pub fn set_operator(&mut self, operator: crate::Operator) {
        self.expr.push_operator(operator.into_operator());
    }

    #[deprecated(note = "Use pushOperator instead")]
    #[wasm_bindgen(js_name = withOperator)]
    pub fn with_operator(self, name: js_sys::JsString, global: JsValue) -> Self {
        unimplemented!("Use the `pushOperator` function instead");
    }

    #[wasm_bindgen(js_name = evaluateStr)]
    pub fn evaluate_str(&self, expression: js_sys::JsString, cx: JsValue) -> JsValue {
        let Some(expr) = expression.as_string() else {
            wasm_bindgen::throw_str("Expression could not be cast to native string");
        };

        *self.global_context.borrow_mut() = js_value_to_object(cx).unwrap_throw();

        let error_message_or_result = self.expr.evaluate(expr)
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

        return value_to_js_object(res)
            .unwrap_throw();
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
                    Literal::Nothing => Token::new("nothing".to_owned(), TokenType::Nothing),
                    Literal::Bool(bool) => Token::new(format!("{}", bool), TokenType::Bool),
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

        flatten(&mut match self.expr.parse(&expr) {
            Ok(value) => value,
            Err(err) => wasm_bindgen::throw_str(&format!("{:#?}", err))
        }).unwrap_or(vec![])
    }

    #[wasm_bindgen(js_name="clone")]
    pub fn clone(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            global_context: self.global_context.clone(),
        }
    }

    #[wasm_bindgen(js_name="provider")]
    pub fn provider(&self) -> JsValue {
        self.expr.provider().inner.clone().into()
    }
}


#[wasm_bindgen]
pub struct Token {
    #[wasm_bindgen(js_name="type")]
    pub token_type: TokenType,
    token: String,
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
    Nothing,
    Num,
    String,
    Bool,
    Address
}