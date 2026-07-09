#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::bar::CandleStore;
use crate::types::IndicatorArena;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FormulaOutputConfig {
    pub name: String,
    pub renderer: String,
    pub pane: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FormulaIndicatorConfig {
    pub name: String,
    pub pane: String,
    #[serde(default)]
    pub params: HashMap<String, f64>,
    pub outputs: Vec<FormulaOutputConfig>,
    pub script: String,
}

pub(crate) struct FormulaIndicator {
    pub id: u32,
    pub name: String,
    pub pane: String,
    pub params: HashMap<String, f64>,
    pub outputs: Vec<FormulaOutputConfig>,
    program: Program,
    runtime: FormulaRuntime,
    pub values: IndicatorArena,
    last_len: usize,
}

impl FormulaIndicator {
    pub(crate) fn new(
        id: u32,
        config: FormulaIndicatorConfig,
        store: &CandleStore,
    ) -> Result<Self, JsValue> {
        if config.outputs.is_empty() {
            return Err(JsValue::from_str(
                "formula indicator must define at least one output",
            ));
        }
        let program = parse_program(&config.script).map_err(js_error)?;
        let mut indicator = Self {
            id,
            name: config.name,
            pane: config.pane,
            params: config.params,
            outputs: config.outputs,
            program,
            runtime: FormulaRuntime::default(),
            values: IndicatorArena::from_outputs(Vec::new()),
            last_len: 0,
        };
        indicator.recompute(store)?;
        Ok(indicator)
    }

    pub(crate) fn recompute(&mut self, store: &CandleStore) -> Result<(), JsValue> {
        self.runtime.reset();
        self.values = IndicatorArena::with_names(self.output_names());
        self.last_len = 0;
        for index in 0..store.len() {
            let row = self.step_row(store, index).map_err(js_error)?;
            self.values.push_row(&row);
            self.last_len += 1;
        }
        Ok(())
    }

    pub(crate) fn update(&mut self, store: &CandleStore) -> Result<(), JsValue> {
        if store.len() == self.last_len + 1 {
            let row = self.step_row(store, store.len() - 1).map_err(js_error)?;
            self.values.push_row(&row);
            self.last_len = store.len();
            return Ok(());
        }
        self.recompute(store)
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs
            .iter()
            .map(|output| output.name.clone())
            .collect()
    }

    fn step_row(&mut self, store: &CandleStore, index: usize) -> Result<Vec<f64>, String> {
        let mut env = HashMap::<String, f64>::new();
        for statement in &self.program.statements {
            match statement {
                Statement::Assign(name, expr) => {
                    let mut ctx = StepEvalContext {
                        store,
                        index,
                        params: &self.params,
                        env: &env,
                        runtime: &mut self.runtime,
                    };
                    let value = eval_expr_step(expr, &mut ctx)?;
                    env.insert(name.clone(), value);
                }
            }
        }
        self.outputs
            .iter()
            .map(|output| {
                env.get(&output.name)
                    .copied()
                    .ok_or_else(|| format!("formula output '{}' was not assigned", output.name))
            })
            .collect()
    }
}

#[derive(Default)]
struct FormulaRuntime {
    calls: HashMap<usize, CallState>,
}

impl FormulaRuntime {
    fn reset(&mut self) {
        self.calls.clear();
    }
}

enum CallState {
    Ema {
        period: usize,
        current: Option<f64>,
    },
    Sma {
        period: usize,
        values: VecDeque<f64>,
        sum: f64,
    },
    Rsi {
        period: usize,
        prev_close: Option<f64>,
        seed_gain: f64,
        seed_loss: f64,
        seed_count: usize,
        avg_gain: f64,
        avg_loss: f64,
        initialized: bool,
    },
    Atr {
        period: usize,
        prev_close: Option<f64>,
        current: Option<f64>,
        tr_sum: f64,
    },
    Window {
        period: usize,
        values: VecDeque<f64>,
    },
    Stdev {
        period: usize,
        values: VecDeque<f64>,
    },
    Cross {
        prev_left: Option<f64>,
        prev_right: Option<f64>,
    },
}

#[derive(Clone, Debug)]
struct Program {
    statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
enum Statement {
    Assign(String, Expr),
}

#[derive(Clone, Debug)]
enum Expr {
    Number(f64),
    Ident(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call {
        id: usize,
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Copy, Debug)]
enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug)]
enum Value {
    Scalar(f64),
    Series(Vec<f64>),
}

impl Value {
    fn into_series(self, len: usize) -> Option<Vec<f64>> {
        match self {
            Value::Scalar(value) => Some(vec![value; len]),
            Value::Series(values) if values.len() == len => Some(values),
            Value::Series(values) if values.len() == 1 => Some(vec![values[0]; len]),
            Value::Series(_) => None,
        }
    }

    fn as_scalar(&self) -> Option<f64> {
        match self {
            Value::Scalar(value) => Some(*value),
            Value::Series(values) if values.len() == 1 => Some(values[0]),
            Value::Series(values)
                if !values.is_empty() && values.iter().all(|value| *value == values[0]) =>
            {
                Some(values[0])
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Ident(String),
    Number(f64),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqEq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
    Not,
    Eof,
}

struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    fn next(&mut self) -> Option<char> {
        self.peeked.take().or_else(|| self.chars.next())
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(ch) if ch.is_ascii_whitespace() || ch == ';') {
                self.next();
            }
            if self.peek() == Some('-') {
                let mut clone = self.chars.clone();
                if matches!(clone.next(), Some('-')) {
                    self.next();
                    self.next();
                    while let Some(ch) = self.peek() {
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                        self.next();
                    }
                    continue;
                }
            }
            break;
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_ws_and_comments();
        let Some(ch) = self.next() else {
            return Ok(Token::Eof);
        };
        let token = match ch {
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.next();
                    Token::EqEq
                } else {
                    Token::Equal
                }
            }
            '~' => {
                if self.peek() == Some('=') {
                    self.next();
                    Token::NotEq
                } else {
                    return Err("unexpected '~'".to_string());
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.next();
                    Token::LessEq
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.next();
                    Token::GreaterEq
                } else {
                    Token::Greater
                }
            }
            '.' | '0'..='9' => {
                let mut text = String::new();
                text.push(ch);
                let mut seen_dot = ch == '.';
                let mut seen_exp = false;
                loop {
                    match self.peek() {
                        Some(next) if next.is_ascii_digit() => text.push(self.next().unwrap()),
                        Some('.') if !seen_dot && !seen_exp => {
                            seen_dot = true;
                            text.push(self.next().unwrap());
                        }
                        Some('e' | 'E') if !seen_exp => {
                            seen_exp = true;
                            text.push(self.next().unwrap());
                            if matches!(self.peek(), Some('+' | '-')) {
                                text.push(self.next().unwrap());
                            }
                        }
                        _ => break,
                    }
                }
                let value = text
                    .parse::<f64>()
                    .map_err(|_| format!("invalid number literal: {text}"))?;
                Token::Number(value)
            }
            '_' | 'a'..='z' | 'A'..='Z' => {
                let mut ident = String::new();
                ident.push(ch);
                while matches!(self.peek(), Some(next) if next == '_' || next.is_ascii_alphanumeric())
                {
                    ident.push(self.next().unwrap());
                }
                match ident.as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    _ => Token::Ident(ident),
                }
            }
            other => return Err(format!("unexpected character: {other}")),
        };
        Ok(token)
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    next_call_id: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self {
            lexer,
            current,
            next_call_id: 0,
        })
    }

    fn bump(&mut self) -> Result<(), String> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while self.current != Token::Eof {
            statements.push(self.parse_assignment()?);
        }
        Ok(Program { statements })
    }

    fn parse_assignment(&mut self) -> Result<Statement, String> {
        let name = match &self.current {
            Token::Ident(name) => name.clone(),
            Token::Eof => return Err("unexpected end of formula".to_string()),
            other => return Err(format!("expected assignment name, found {other:?}")),
        };
        self.bump()?;
        match self.current {
            Token::Equal => self.bump()?,
            _ => return Err(format!("expected '=' after assignment name '{name}'")),
        }
        let expr = self.parse_expr()?;
        Ok(Statement::Assign(name, expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_and()?;
        while self.current == Token::Or {
            self.bump()?;
            let right = self.parse_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        while self.current == Token::And {
            self.bump()?;
            let right = self.parse_comparison()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        loop {
            let op = match self.current {
                Token::EqEq => BinaryOp::Eq,
                Token::NotEq => BinaryOp::Ne,
                Token::Less => BinaryOp::Lt,
                Token::LessEq => BinaryOp::Le,
                Token::Greater => BinaryOp::Gt,
                Token::GreaterEq => BinaryOp::Ge,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = match self.current {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.current {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Rem,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current {
            Token::Minus => {
                self.bump()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary()?)))
            }
            Token::Not => {
                self.bump()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()?)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match &self.current {
            Token::Number(value) => {
                let value = *value;
                self.bump()?;
                Ok(Expr::Number(value))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.bump()?;
                if self.current == Token::LParen {
                    self.bump()?;
                    let mut args = Vec::new();
                    if self.current != Token::RParen {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.current == Token::Comma {
                                self.bump()?;
                                continue;
                            }
                            break;
                        }
                    }
                    if self.current != Token::RParen {
                        return Err(format!("expected ')' after call to {name}"));
                    }
                    self.bump()?;
                    let id = self.next_call_id;
                    self.next_call_id += 1;
                    Ok(Expr::Call { id, name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::LParen => {
                self.bump()?;
                let expr = self.parse_expr()?;
                if self.current != Token::RParen {
                    return Err("expected ')'".to_string());
                }
                self.bump()?;
                Ok(expr)
            }
            other => Err(format!("unexpected token in expression: {other:?}")),
        }
    }
}

fn parse_program(input: &str) -> Result<Program, String> {
    Parser::new(input)?.parse_program()
}

fn evaluate_program(
    program: &Program,
    store: &CandleStore,
    params: &HashMap<String, f64>,
) -> Result<HashMap<String, Value>, String> {
    let mut env = HashMap::<String, Value>::new();
    for statement in &program.statements {
        match statement {
            Statement::Assign(name, expr) => {
                let value = eval_expr(expr, store, params, &env)?;
                env.insert(name.clone(), value);
            }
        }
    }
    Ok(env)
}

fn eval_expr(
    expr: &Expr,
    store: &CandleStore,
    params: &HashMap<String, f64>,
    env: &HashMap<String, Value>,
) -> Result<Value, String> {
    match expr {
        Expr::Number(value) => Ok(Value::Scalar(*value)),
        Expr::Ident(name) => resolve_ident(name, store, params, env),
        Expr::Unary(op, expr) => {
            let value = eval_expr(expr, store, params, env)?;
            match op {
                UnaryOp::Neg => unary_numeric(value, |x| -x),
                UnaryOp::Not => unary_truthy(value, |x| if x { 0.0 } else { 1.0 }),
            }
        }
        Expr::Binary(left, op, right) => {
            let left = eval_expr(left, store, params, env)?;
            let right = eval_expr(right, store, params, env)?;
            eval_binary(left, *op, right)
        }
        Expr::Call { name, args, .. } => eval_call(name, args, store, params, env),
    }
}

struct StepEvalContext<'a> {
    store: &'a CandleStore,
    index: usize,
    params: &'a HashMap<String, f64>,
    env: &'a HashMap<String, f64>,
    runtime: &'a mut FormulaRuntime,
}

fn eval_expr_step(expr: &Expr, ctx: &mut StepEvalContext<'_>) -> Result<f64, String> {
    match expr {
        Expr::Number(value) => Ok(*value),
        Expr::Ident(name) => resolve_ident_step(name, ctx),
        Expr::Unary(op, expr) => {
            let value = eval_expr_step(expr, ctx)?;
            match op {
                UnaryOp::Neg => Ok(-value),
                UnaryOp::Not => Ok(if truthy(value) { 0.0 } else { 1.0 }),
            }
        }
        Expr::Binary(left, op, right) => {
            let left = eval_expr_step(left, ctx)?;
            let right = eval_expr_step(right, ctx)?;
            Ok(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Sub => left - right,
                BinaryOp::Mul => left * right,
                BinaryOp::Div => left / right,
                BinaryOp::Rem => left % right,
                BinaryOp::Eq => bool_to_num(left == right),
                BinaryOp::Ne => bool_to_num(left != right),
                BinaryOp::Lt => bool_to_num(left < right),
                BinaryOp::Le => bool_to_num(left <= right),
                BinaryOp::Gt => bool_to_num(left > right),
                BinaryOp::Ge => bool_to_num(left >= right),
                BinaryOp::And => bool_to_num(truthy(left) && truthy(right)),
                BinaryOp::Or => bool_to_num(truthy(left) || truthy(right)),
            })
        }
        Expr::Call { id, name, args } => eval_call_step(*id, name, args, ctx),
    }
}

fn resolve_ident_step(name: &str, ctx: &StepEvalContext<'_>) -> Result<f64, String> {
    if let Some(value) = ctx.env.get(name) {
        return Ok(*value);
    }
    if let Some(value) = ctx.params.get(name) {
        return Ok(*value);
    }
    match name {
        "open" => Ok(ctx.store.open[ctx.index]),
        "high" => Ok(ctx.store.high[ctx.index]),
        "low" => Ok(ctx.store.low[ctx.index]),
        "close" => Ok(ctx.store.close[ctx.index]),
        "volume" => Ok(ctx.store.volume[ctx.index]),
        "time" => Ok(ctx.store.time[ctx.index] as f64),
        "hl2" => Ok((ctx.store.high[ctx.index] + ctx.store.low[ctx.index]) / 2.0),
        "hlc3" => Ok((ctx.store.high[ctx.index]
            + ctx.store.low[ctx.index]
            + ctx.store.close[ctx.index])
            / 3.0),
        "ohlc4" => Ok((ctx.store.open[ctx.index]
            + ctx.store.high[ctx.index]
            + ctx.store.low[ctx.index]
            + ctx.store.close[ctx.index])
            / 4.0),
        "na" | "nil" => Ok(f64::NAN),
        "true" => Ok(1.0),
        "false" => Ok(0.0),
        _ => Err(format!("unknown identifier: {name}")),
    }
}

fn resolve_ident(
    name: &str,
    store: &CandleStore,
    params: &HashMap<String, f64>,
    env: &HashMap<String, Value>,
) -> Result<Value, String> {
    if let Some(value) = env.get(name) {
        return Ok(value.clone());
    }
    if let Some(value) = params.get(name) {
        return Ok(Value::Scalar(*value));
    }
    match name {
        "open" => Ok(Value::Series(store.open.to_vec())),
        "high" => Ok(Value::Series(store.high.to_vec())),
        "low" => Ok(Value::Series(store.low.to_vec())),
        "close" => Ok(Value::Series(store.close.to_vec())),
        "volume" => Ok(Value::Series(store.volume.to_vec())),
        "time" => Ok(Value::Series(
            store.time.iter().map(|v| *v as f64).collect(),
        )),
        "hl2" => Ok(Value::Series(
            store
                .high
                .iter()
                .zip(store.low.iter())
                .map(|(high, low)| (high + low) / 2.0)
                .collect(),
        )),
        "hlc3" => Ok(Value::Series(
            store
                .high
                .iter()
                .zip(store.low.iter())
                .zip(store.close.iter())
                .map(|((high, low), close)| (high + low + close) / 3.0)
                .collect(),
        )),
        "ohlc4" => Ok(Value::Series(
            store
                .open
                .iter()
                .zip(store.high.iter())
                .zip(store.low.iter())
                .zip(store.close.iter())
                .map(|(((open, high), low), close)| (open + high + low + close) / 4.0)
                .collect(),
        )),
        "na" | "nil" => Ok(Value::Scalar(f64::NAN)),
        "true" => Ok(Value::Scalar(1.0)),
        "false" => Ok(Value::Scalar(0.0)),
        _ => Err(format!("unknown identifier: {name}")),
    }
}

fn eval_binary(left: Value, op: BinaryOp, right: Value) -> Result<Value, String> {
    use BinaryOp::*;
    match op {
        Add | Sub | Mul | Div | Rem => eval_binary_numeric(left, right, |a, b| match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            Rem => a % b,
            _ => unreachable!(),
        }),
        Eq | Ne | Lt | Le | Gt | Ge => eval_binary_cmp(left, right, |a, b| match op {
            Eq => a == b,
            Ne => a != b,
            Lt => a < b,
            Le => a <= b,
            Gt => a > b,
            Ge => a >= b,
            _ => unreachable!(),
        }),
        And | Or => eval_binary_bool(left, right, op == And),
    }
}

fn eval_binary_numeric(
    left: Value,
    right: Value,
    op: impl Fn(f64, f64) -> f64 + Copy,
) -> Result<Value, String> {
    match (left, right) {
        (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(op(a, b))),
        (Value::Series(a), Value::Scalar(b)) => {
            Ok(Value::Series(a.into_iter().map(|x| op(x, b)).collect()))
        }
        (Value::Scalar(a), Value::Series(b)) => {
            Ok(Value::Series(b.into_iter().map(|x| op(a, x)).collect()))
        }
        (Value::Series(a), Value::Series(b)) => {
            if a.len() != b.len() {
                return Err("series lengths do not match".to_string());
            }
            Ok(Value::Series(
                a.into_iter().zip(b).map(|(x, y)| op(x, y)).collect(),
            ))
        }
    }
}

fn eval_binary_cmp(
    left: Value,
    right: Value,
    cmp: impl Fn(f64, f64) -> bool + Copy,
) -> Result<Value, String> {
    match (left, right) {
        (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(bool_to_num(cmp(a, b)))),
        (Value::Series(a), Value::Scalar(b)) => Ok(Value::Series(
            a.into_iter().map(|x| bool_to_num(cmp(x, b))).collect(),
        )),
        (Value::Scalar(a), Value::Series(b)) => Ok(Value::Series(
            b.into_iter().map(|x| bool_to_num(cmp(a, x))).collect(),
        )),
        (Value::Series(a), Value::Series(b)) => {
            if a.len() != b.len() {
                return Err("series lengths do not match".to_string());
            }
            Ok(Value::Series(
                a.into_iter()
                    .zip(b)
                    .map(|(x, y)| bool_to_num(cmp(x, y)))
                    .collect(),
            ))
        }
    }
}

fn eval_binary_bool(left: Value, right: Value, is_and: bool) -> Result<Value, String> {
    match (left, right) {
        (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(bool_to_num(if is_and {
            truthy(a) && truthy(b)
        } else {
            truthy(a) || truthy(b)
        }))),
        (Value::Series(a), Value::Scalar(b)) => Ok(Value::Series(
            a.into_iter()
                .map(|x| {
                    bool_to_num(if is_and {
                        truthy(x) && truthy(b)
                    } else {
                        truthy(x) || truthy(b)
                    })
                })
                .collect(),
        )),
        (Value::Scalar(a), Value::Series(b)) => Ok(Value::Series(
            b.into_iter()
                .map(|x| {
                    bool_to_num(if is_and {
                        truthy(a) && truthy(x)
                    } else {
                        truthy(a) || truthy(x)
                    })
                })
                .collect(),
        )),
        (Value::Series(a), Value::Series(b)) => {
            if a.len() != b.len() {
                return Err("series lengths do not match".to_string());
            }
            Ok(Value::Series(
                a.into_iter()
                    .zip(b)
                    .map(|(x, y)| {
                        bool_to_num(if is_and {
                            truthy(x) && truthy(y)
                        } else {
                            truthy(x) || truthy(y)
                        })
                    })
                    .collect(),
            ))
        }
    }
}

fn eval_call(
    name: &str,
    args: &[Expr],
    store: &CandleStore,
    params: &HashMap<String, f64>,
    env: &HashMap<String, Value>,
) -> Result<Value, String> {
    match name {
        "sma" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(sma_series(&values, period)))
        }
        "ema" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(ema_series(&values, period)))
        }
        "rsi" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(rsi_series(&values, period)))
        }
        "atr" => {
            let period = eval_usize_arg(args.first(), store, params, env)?;
            Ok(Value::Series(atr_series(store, period)))
        }
        "highest" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(highest_series(&values, period)))
        }
        "lowest" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(lowest_series(&values, period)))
        }
        "stdev" => {
            let values = eval_series_arg(args.first(), store, params, env)?;
            let period = eval_usize_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(stdev_series(&values, period)))
        }
        "abs" => {
            let value = eval_expr(
                args.first()
                    .ok_or_else(|| "abs() requires one argument".to_string())?,
                store,
                params,
                env,
            )?;
            unary_numeric(value, f64::abs)
        }
        "min" => {
            let left = eval_expr(
                args.first()
                    .ok_or_else(|| "min() requires two arguments".to_string())?,
                store,
                params,
                env,
            )?;
            let right = eval_expr(
                args.get(1)
                    .ok_or_else(|| "min() requires two arguments".to_string())?,
                store,
                params,
                env,
            )?;
            eval_binary_numeric(left, right, f64::min)
        }
        "max" => {
            let left = eval_expr(
                args.first()
                    .ok_or_else(|| "max() requires two arguments".to_string())?,
                store,
                params,
                env,
            )?;
            let right = eval_expr(
                args.get(1)
                    .ok_or_else(|| "max() requires two arguments".to_string())?,
                store,
                params,
                env,
            )?;
            eval_binary_numeric(left, right, f64::max)
        }
        "iff" => {
            let cond = eval_expr(
                args.first()
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                store,
                params,
                env,
            )?;
            let yes = eval_expr(
                args.get(1)
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                store,
                params,
                env,
            )?;
            let no = eval_expr(
                args.get(2)
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                store,
                params,
                env,
            )?;
            eval_iff(cond, yes, no)
        }
        "nz" => {
            let value = eval_expr(
                args.first()
                    .ok_or_else(|| "nz() requires one or two arguments".to_string())?,
                store,
                params,
                env,
            )?;
            let fallback = if let Some(expr) = args.get(1) {
                eval_expr(expr, store, params, env)?
            } else {
                Value::Scalar(0.0)
            };
            eval_nz(value, fallback)
        }
        "cross" | "crossover" | "crossunder" => {
            let left = eval_series_arg(args.first(), store, params, env)?;
            let right = eval_series_arg(args.get(1), store, params, env)?;
            Ok(Value::Series(match name {
                "cross" => cross_series(&left, &right),
                "crossover" => crossover_series(&left, &right),
                _ => crossunder_series(&left, &right),
            }))
        }
        _ => Err(format!("unknown function: {name}")),
    }
}

fn eval_call_step(
    id: usize,
    name: &str,
    args: &[Expr],
    ctx: &mut StepEvalContext<'_>,
) -> Result<f64, String> {
    match name {
        "sma" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "sma() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Sma {
                    period,
                    values: VecDeque::new(),
                    sum: 0.0,
                });
            sma_step(state, value, period)
        }
        "ema" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "ema() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Ema {
                    period,
                    current: None,
                });
            ema_step(state, value, period)
        }
        "rsi" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "rsi() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Rsi {
                    period,
                    prev_close: None,
                    seed_gain: 0.0,
                    seed_loss: 0.0,
                    seed_count: 0,
                    avg_gain: 0.0,
                    avg_loss: 0.0,
                    initialized: false,
                });
            rsi_step(state, value, period)
        }
        "atr" => {
            let period = eval_usize_arg_step(args.first(), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Atr {
                    period,
                    prev_close: None,
                    current: None,
                    tr_sum: 0.0,
                });
            atr_step(state, ctx.store, ctx.index, period)
        }
        "highest" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "highest() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Window {
                    period,
                    values: VecDeque::new(),
                });
            window_step_max(state, value, period)
        }
        "lowest" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "lowest() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Window {
                    period,
                    values: VecDeque::new(),
                });
            window_step_min(state, value, period)
        }
        "stdev" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "stdev() requires two arguments".to_string())?,
                ctx,
            )?;
            let period = eval_usize_arg_step(args.get(1), ctx)?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Stdev {
                    period,
                    values: VecDeque::new(),
                });
            stdev_step(state, value, period)
        }
        "abs" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "abs() requires one argument".to_string())?,
                ctx,
            )?;
            Ok(value.abs())
        }
        "min" => {
            let left = eval_expr_step(
                args.first()
                    .ok_or_else(|| "min() requires two arguments".to_string())?,
                ctx,
            )?;
            let right = eval_expr_step(
                args.get(1)
                    .ok_or_else(|| "min() requires two arguments".to_string())?,
                ctx,
            )?;
            Ok(left.min(right))
        }
        "max" => {
            let left = eval_expr_step(
                args.first()
                    .ok_or_else(|| "max() requires two arguments".to_string())?,
                ctx,
            )?;
            let right = eval_expr_step(
                args.get(1)
                    .ok_or_else(|| "max() requires two arguments".to_string())?,
                ctx,
            )?;
            Ok(left.max(right))
        }
        "iff" => {
            let cond = eval_expr_step(
                args.first()
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                ctx,
            )?;
            let yes = eval_expr_step(
                args.get(1)
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                ctx,
            )?;
            let no = eval_expr_step(
                args.get(2)
                    .ok_or_else(|| "iff() requires three arguments".to_string())?,
                ctx,
            )?;
            Ok(if truthy(cond) { yes } else { no })
        }
        "nz" => {
            let value = eval_expr_step(
                args.first()
                    .ok_or_else(|| "nz() requires one or two arguments".to_string())?,
                ctx,
            )?;
            let fallback = if let Some(expr) = args.get(1) {
                eval_expr_step(expr, ctx)?
            } else {
                0.0
            };
            Ok(if value.is_nan() { fallback } else { value })
        }
        "cross" | "crossover" | "crossunder" => {
            let left = eval_expr_step(
                args.first()
                    .ok_or_else(|| "cross() requires two arguments".to_string())?,
                ctx,
            )?;
            let right = eval_expr_step(
                args.get(1)
                    .ok_or_else(|| "cross() requires two arguments".to_string())?,
                ctx,
            )?;
            let state = ctx
                .runtime
                .calls
                .entry(id)
                .or_insert_with(|| CallState::Cross {
                    prev_left: None,
                    prev_right: None,
                });
            cross_step(state, name, left, right)
        }
        _ => Err(format!("unknown function: {name}")),
    }
}

fn eval_usize_arg_step(arg: Option<&Expr>, ctx: &mut StepEvalContext<'_>) -> Result<usize, String> {
    let expr = arg.ok_or_else(|| "missing numeric argument".to_string())?;
    let value = eval_expr_step(expr, ctx)?;
    if !value.is_finite() || value <= 0.0 {
        return Err("numeric argument must be a positive finite number".to_string());
    }
    Ok(value.round() as usize)
}

fn sma_step(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Sma {
        period: state_period,
        values,
        sum,
    } = state
    else {
        return Err("sma state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic SMA period is not supported".to_string());
    }
    values.push_back(value);
    *sum += value;
    if values.len() > period {
        *sum -= values.pop_front().unwrap_or(0.0);
    }
    if values.len() < period {
        Ok(f64::NAN)
    } else {
        Ok(*sum / period as f64)
    }
}

fn ema_step(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Ema {
        period: state_period,
        current,
    } = state
    else {
        return Err("ema state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic EMA period is not supported".to_string());
    }
    if period == 0 {
        return Ok(value);
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let next = match *current {
        Some(previous) => alpha * value + (1.0 - alpha) * previous,
        None => value,
    };
    *current = Some(next);
    Ok(next)
}

fn rsi_step(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Rsi {
        period: state_period,
        prev_close,
        seed_gain,
        seed_loss,
        seed_count,
        avg_gain,
        avg_loss,
        initialized,
    } = state
    else {
        return Err("rsi state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic RSI period is not supported".to_string());
    }
    let Some(previous_close) = *prev_close else {
        *prev_close = Some(value);
        return Ok(f64::NAN);
    };
    let delta = value - previous_close;
    *prev_close = Some(value);
    if !*initialized {
        if delta >= 0.0 {
            *seed_gain += delta;
        } else {
            *seed_loss += -delta;
        }
        *seed_count += 1;
        if *seed_count < period {
            return Ok(f64::NAN);
        }
        *avg_gain = *seed_gain / period as f64;
        *avg_loss = *seed_loss / period as f64;
        *initialized = true;
        return Ok(rsi_from_avgs(*avg_gain, *avg_loss));
    }
    let gain = delta.max(0.0);
    let loss = (-delta).max(0.0);
    *avg_gain = (*avg_gain * (period as f64 - 1.0) + gain) / period as f64;
    *avg_loss = (*avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    Ok(rsi_from_avgs(*avg_gain, *avg_loss))
}

fn atr_step(
    state: &mut CallState,
    store: &CandleStore,
    index: usize,
    period: usize,
) -> Result<f64, String> {
    let CallState::Atr {
        period: state_period,
        prev_close,
        current,
        tr_sum,
    } = state
    else {
        return Err("atr state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic ATR period is not supported".to_string());
    }
    let tr = true_range(store, index);
    if index == 0 {
        *prev_close = Some(store.close[0]);
        return Ok(f64::NAN);
    }
    *prev_close = Some(store.close[index]);
    if index < period {
        *tr_sum += tr;
        return Ok(f64::NAN);
    }
    if index == period {
        *tr_sum += tr;
        let value = *tr_sum / period as f64;
        *current = Some(value);
        return Ok(value);
    }
    let prev = current.ok_or_else(|| "atr state not initialized".to_string())?;
    let next = (prev * (period - 1) as f64 + tr) / period as f64;
    *current = Some(next);
    Ok(next)
}

fn window_step_max(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Window {
        period: state_period,
        values,
    } = state
    else {
        return Err("highest state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic window period is not supported".to_string());
    }
    values.push_back(value);
    if values.len() > period {
        values.pop_front();
    }
    if values.len() < period {
        return Ok(f64::NAN);
    }
    Ok(values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

fn window_step_min(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Window {
        period: state_period,
        values,
    } = state
    else {
        return Err("lowest state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic window period is not supported".to_string());
    }
    values.push_back(value);
    if values.len() > period {
        values.pop_front();
    }
    if values.len() < period {
        return Ok(f64::NAN);
    }
    Ok(values.iter().copied().fold(f64::INFINITY, f64::min))
}

fn stdev_step(state: &mut CallState, value: f64, period: usize) -> Result<f64, String> {
    let CallState::Stdev {
        period: state_period,
        values,
    } = state
    else {
        return Err("stdev state mismatch".to_string());
    };
    if *state_period != period {
        return Err("dynamic stdev period is not supported".to_string());
    }
    values.push_back(value);
    if values.len() > period {
        values.pop_front();
    }
    if values.len() < period {
        return Ok(f64::NAN);
    }
    let mean = values.iter().sum::<f64>() / period as f64;
    let variance = values
        .iter()
        .map(|v| {
            let delta = v - mean;
            delta * delta
        })
        .sum::<f64>()
        / period as f64;
    Ok(variance.sqrt())
}

fn cross_step(state: &mut CallState, name: &str, left: f64, right: f64) -> Result<f64, String> {
    let CallState::Cross {
        prev_left,
        prev_right,
    } = state
    else {
        return Err("cross state mismatch".to_string());
    };
    let result = match (*prev_left, *prev_right) {
        (Some(prev_left), Some(prev_right)) => match name {
            "cross" => bool_to_num(
                (prev_left <= prev_right && left > right)
                    || (prev_left >= prev_right && left < right),
            ),
            "crossover" => bool_to_num(prev_left <= prev_right && left > right),
            "crossunder" => bool_to_num(prev_left >= prev_right && left < right),
            _ => unreachable!(),
        },
        _ => f64::NAN,
    };
    *prev_left = Some(left);
    *prev_right = Some(right);
    Ok(result)
}

fn eval_series_arg(
    arg: Option<&Expr>,
    store: &CandleStore,
    params: &HashMap<String, f64>,
    env: &HashMap<String, Value>,
) -> Result<Vec<f64>, String> {
    let expr = arg.ok_or_else(|| "missing series argument".to_string())?;
    match eval_expr(expr, store, params, env)? {
        Value::Series(values) => Ok(values),
        Value::Scalar(value) => Ok(vec![value; store.len()]),
    }
}

fn eval_usize_arg(
    arg: Option<&Expr>,
    store: &CandleStore,
    params: &HashMap<String, f64>,
    env: &HashMap<String, Value>,
) -> Result<usize, String> {
    let expr = arg.ok_or_else(|| "missing numeric argument".to_string())?;
    let value = eval_expr(expr, store, params, env)?;
    let scalar = value
        .as_scalar()
        .ok_or_else(|| "expected a scalar numeric argument".to_string())?;
    if !scalar.is_finite() || scalar <= 0.0 {
        return Err("numeric argument must be a positive finite number".to_string());
    }
    Ok(scalar.round() as usize)
}

fn unary_numeric(value: Value, op: impl Fn(f64) -> f64 + Copy) -> Result<Value, String> {
    match value {
        Value::Scalar(value) => Ok(Value::Scalar(op(value))),
        Value::Series(values) => Ok(Value::Series(values.into_iter().map(op).collect())),
    }
}

fn unary_truthy(value: Value, op: impl Fn(bool) -> f64 + Copy) -> Result<Value, String> {
    match value {
        Value::Scalar(value) => Ok(Value::Scalar(op(truthy(value)))),
        Value::Series(values) => Ok(Value::Series(
            values.into_iter().map(|value| op(truthy(value))).collect(),
        )),
    }
}

fn eval_iff(cond: Value, yes: Value, no: Value) -> Result<Value, String> {
    match cond {
        Value::Scalar(cond) => {
            if truthy(cond) {
                Ok(yes)
            } else {
                Ok(no)
            }
        }
        Value::Series(cond) => {
            let len = cond.len();
            let yes = yes
                .into_series(len)
                .ok_or_else(|| "iff() branches must match the condition length".to_string())?;
            let no = no
                .into_series(len)
                .ok_or_else(|| "iff() branches must match the condition length".to_string())?;
            Ok(Value::Series(
                cond.into_iter()
                    .zip(yes)
                    .zip(no)
                    .map(|((cond, yes), no)| if truthy(cond) { yes } else { no })
                    .collect(),
            ))
        }
    }
}

fn eval_nz(value: Value, fallback: Value) -> Result<Value, String> {
    match value {
        Value::Scalar(value) => {
            if value.is_nan() {
                Ok(fallback)
            } else {
                Ok(Value::Scalar(value))
            }
        }
        Value::Series(values) => {
            let fallback = fallback
                .into_series(values.len())
                .ok_or_else(|| "nz() fallback must match the input length".to_string())?;
            Ok(Value::Series(
                values
                    .into_iter()
                    .zip(fallback)
                    .map(|(value, fallback)| if value.is_nan() { fallback } else { value })
                    .collect(),
            ))
        }
    }
}

fn sma_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let mut sum = 0.0;
    for (index, value) in values.iter().copied().enumerate() {
        sum += value;
        if index >= period {
            sum -= values[index - period];
        }
        if index + 1 >= period {
            out[index] = sum / period as f64;
        }
    }
    out
}

fn ema_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if values.is_empty() {
        return out;
    }
    if period == 0 {
        return values.to_vec();
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    for (index, value) in values.iter().copied().enumerate() {
        if value.is_nan() {
            continue;
        }
        let next = match current {
            Some(previous) => alpha * value + (1.0 - alpha) * previous,
            None => value,
        };
        current = Some(next);
        out[index] = next;
    }
    out
}

fn rsi_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() <= period {
        return out;
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for index in 1..=period {
        let delta = values[index] - values[index - 1];
        if delta >= 0.0 {
            gains += delta;
        } else {
            losses += -delta;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    out[period] = rsi_from_avgs(avg_gain, avg_loss);
    for index in (period + 1)..values.len() {
        let delta = values[index] - values[index - 1];
        let gain = delta.max(0.0);
        let loss = (-delta).max(0.0);
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
        out[index] = rsi_from_avgs(avg_gain, avg_loss);
    }
    out
}

fn rsi_from_avgs(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        return 100.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn atr_series(store: &CandleStore, period: usize) -> Vec<f64> {
    let len = store.len();
    let mut out = vec![f64::NAN; len];
    if period == 0 || len <= period {
        return out;
    }
    let mut tr = Vec::with_capacity(len);
    for index in 0..len {
        tr.push(true_range(store, index));
    }
    let mut current = tr[1..=period].iter().sum::<f64>() / period as f64;
    out[period] = current;
    for index in (period + 1)..len {
        current = (current * (period - 1) as f64 + tr[index]) / period as f64;
        out[index] = current;
    }
    out
}

fn true_range(store: &CandleStore, index: usize) -> f64 {
    if index == 0 {
        return store.high[0] - store.low[0];
    }
    let previous_close = store.close[index - 1];
    (store.high[index] - store.low[index])
        .max((store.high[index] - previous_close).abs())
        .max((store.low[index] - previous_close).abs())
}

fn highest_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in (period - 1)..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|v| v.is_nan()) {
            continue;
        }
        out[index] = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }
    out
}

fn lowest_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in (period - 1)..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|v| v.is_nan()) {
            continue;
        }
        out[index] = window.iter().copied().fold(f64::INFINITY, f64::min);
    }
    out
}

fn stdev_series(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in (period - 1)..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|v| v.is_nan()) {
            continue;
        }
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|v| {
                let delta = v - mean;
                delta * delta
            })
            .sum::<f64>()
            / period as f64;
        out[index] = variance.sqrt();
    }
    out
}

fn cross_series(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; left.len()];
    for index in 1..left.len() {
        let prev_left = left[index - 1];
        let prev_right = right[index - 1];
        let curr_left = left[index];
        let curr_right = right[index];
        if prev_left.is_nan() || prev_right.is_nan() || curr_left.is_nan() || curr_right.is_nan() {
            continue;
        }
        out[index] = bool_to_num(
            (prev_left <= prev_right && curr_left > curr_right)
                || (prev_left >= prev_right && curr_left < curr_right),
        );
    }
    out
}

fn crossover_series(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; left.len()];
    for index in 1..left.len() {
        let prev_left = left[index - 1];
        let prev_right = right[index - 1];
        let curr_left = left[index];
        let curr_right = right[index];
        if prev_left.is_nan() || prev_right.is_nan() || curr_left.is_nan() || curr_right.is_nan() {
            continue;
        }
        out[index] = bool_to_num(prev_left <= prev_right && curr_left > curr_right);
    }
    out
}

fn crossunder_series(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; left.len()];
    for index in 1..left.len() {
        let prev_left = left[index - 1];
        let prev_right = right[index - 1];
        let curr_left = left[index];
        let curr_right = right[index];
        if prev_left.is_nan() || prev_right.is_nan() || curr_left.is_nan() || curr_right.is_nan() {
            continue;
        }
        out[index] = bool_to_num(prev_left >= prev_right && curr_left < curr_right);
    }
    out
}

fn truthy(value: f64) -> bool {
    !value.is_nan() && value != 0.0
}

fn bool_to_num(value: bool) -> f64 {
    if value {
        1.0
    } else {
        0.0
    }
}

fn js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bar;

    fn store_from_closes(values: &[f64]) -> CandleStore {
        let len = values.len();
        CandleStore::from_raw_columns(
            (0..len as u32).collect(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            values.to_vec(),
            vec![1.0; len],
        )
    }

    fn assert_series_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert!((actual - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn formula_indicator_exports_only_explicit_outputs() {
        let store = store_from_closes(&[10.0, 12.0, 14.0, 13.0, 15.0]);
        let config = FormulaIndicatorConfig {
            name: "trend".to_string(),
            pane: "separate".to_string(),
            params: HashMap::from([("fast".to_string(), 2.0), ("slow".to_string(), 3.0)]),
            outputs: vec![
                FormulaOutputConfig {
                    name: "spread".to_string(),
                    renderer: "line".to_string(),
                    pane: "separate".to_string(),
                    color: "#2563eb".to_string(),
                },
                FormulaOutputConfig {
                    name: "histogram".to_string(),
                    renderer: "histogram".to_string(),
                    pane: "separate".to_string(),
                    color: "#f59e0b".to_string(),
                },
            ],
            script: r#"
                fast_ma = ema(close, fast)
                slow_ma = ema(close, slow)
                spread = fast_ma - slow_ma
                signal_line = ema(spread, 2)
                histogram = spread - signal_line
            "#
            .to_string(),
        };
        let indicator = FormulaIndicator::new(1, config, &store).unwrap();
        let spread = indicator.values.get_slot(0).unwrap();
        let histogram = indicator.values.get_slot(1).unwrap();
        let fast_ma = ema_series(&store.close, 2);
        let slow_ma = ema_series(&store.close, 3);
        let spread_expected: Vec<f64> = fast_ma
            .iter()
            .zip(slow_ma.iter())
            .map(|(fast, slow)| fast - slow)
            .collect();

        assert_eq!(indicator.name, "trend");
        assert_eq!(indicator.pane, "separate");
        assert!(indicator.values.slot_index("fast_ma").is_none());
        assert!(indicator.values.slot_index("slow_ma").is_none());
        assert_eq!(spread.len(), store.len());
        assert_eq!(histogram.len(), store.len());
        assert_series_close(spread, &spread_expected);
        assert!(histogram.iter().any(|value| value.is_finite()));
    }

    #[test]
    fn formula_indicator_full_load_populates_all_rows() {
        let store = store_from_closes(&[10.0, 12.0, 14.0, 13.0, 15.0]);
        let config = FormulaIndicatorConfig {
            name: "trend".to_string(),
            pane: "separate".to_string(),
            params: HashMap::from([("fast".to_string(), 2.0), ("slow".to_string(), 3.0)]),
            outputs: vec![FormulaOutputConfig {
                name: "spread".to_string(),
                renderer: "line".to_string(),
                pane: "separate".to_string(),
                color: "#2563eb".to_string(),
            }],
            script: r#"
                fast_ma = ema(close, fast)
                slow_ma = ema(close, slow)
                spread = fast_ma - slow_ma
            "#
            .to_string(),
        };

        let indicator = FormulaIndicator::new(1, config, &store).unwrap();
        let spread = indicator.values.get_slot(0).unwrap();
        let fast_ma = ema_series(&store.close, 2);
        let slow_ma = ema_series(&store.close, 3);
        let expected_spread: Vec<f64> = fast_ma
            .iter()
            .zip(slow_ma.iter())
            .map(|(fast, slow)| fast - slow)
            .collect();

        assert_eq!(spread.len(), store.len());
        assert_series_close(spread, &expected_spread);
    }

    #[test]
    fn formula_helpers_match_manual_series() {
        let store = store_from_closes(&[10.0, 12.0, 14.0, 13.0, 15.0]);
        assert_series_close(
            &sma_series(&store.close, 3),
            &[f64::NAN, f64::NAN, 12.0, 13.0, 14.0],
        );
        assert_series_close(
            &ema_series(&store.close, 3),
            &[10.0, 11.0, 12.5, 12.75, 13.875],
        );
        assert_eq!(
            eval_usize_arg(
                Some(&Expr::Number(3.0)),
                &store,
                &HashMap::new(),
                &HashMap::new()
            )
            .unwrap(),
            3
        );
        assert!(atr_series(&store, 3)[3].is_finite());
    }

    #[test]
    fn formula_indicator_incremental_update_matches_full_recompute() {
        let mut store = store_from_closes(&[10.0, 12.0, 14.0, 13.0, 15.0]);
        let config = FormulaIndicatorConfig {
            name: "trend".to_string(),
            pane: "separate".to_string(),
            params: HashMap::from([("fast".to_string(), 2.0), ("slow".to_string(), 3.0)]),
            outputs: vec![FormulaOutputConfig {
                name: "spread".to_string(),
                renderer: "line".to_string(),
                pane: "separate".to_string(),
                color: "#2563eb".to_string(),
            }],
            script: r#"
                fast_ma = ema(close, fast)
                slow_ma = ema(close, slow)
                spread = fast_ma - slow_ma
            "#
            .to_string(),
        };

        let mut indicator = FormulaIndicator::new(1, config.clone(), &store).unwrap();
        assert_eq!(indicator.values.get_slot(0).unwrap().len(), store.len());
        store.push(Bar {
            time: 5,
            open: 16.0,
            high: 17.0,
            low: 15.0,
            close: 16.0,
            volume: 1.0,
        });
        indicator.update(&store).unwrap();
        assert_eq!(indicator.values.get_slot(0).unwrap().len(), store.len());

        let mut rebuilt = FormulaIndicator::new(2, config, &store).unwrap();
        assert_series_close(
            indicator.values.get_slot(0).unwrap(),
            rebuilt.values.get_slot(0).unwrap(),
        );
        rebuilt.update(&store).unwrap();
        assert_series_close(
            indicator.values.get_slot(0).unwrap(),
            rebuilt.values.get_slot(0).unwrap(),
        );
    }
}
