// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Expression {
    Invalid,
    Number(f64),
    X,
    Operation(Box<Expression>, Operation, Box<Expression>),
}

impl Expression {
    pub fn calc(&self, x: f64) -> f64 {
        match self {
            Expression::Invalid => unreachable!(),
            Expression::Number(value) => value.clone(),
            Expression::X => x,
            Expression::Operation(lhs, op, rhs) => {
                let lhs = lhs.calc(x);
                let rhs = rhs.calc(x);

                match op {
                    Operation::Add => lhs + rhs,
                    Operation::Sub => lhs - rhs,
                    Operation::Mul => lhs * rhs,
                    Operation::Div => lhs / rhs,
                    Operation::Pow => lhs.powf(rhs),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, PartialEq)]
enum Token {
    Unknown,
    X,
    Number(f64),
    Operation(Operation),
    BracketOpen,
    BracketClosed,
}

#[tauri::command]
fn parse(function: String) -> Expression {
    let mut tokens = tokenize(function.chars().collect());
    if tokens.contains(&Token::Unknown) { return Expression::Invalid }

    parse_term(&mut tokens)
}

fn tokenize(mut function: Vec<char>) -> Vec<Token> {
    let mut tokens = vec![];
    while !function.is_empty() {
        let letter = function.remove(0);
        let token = match letter {
            ' ' => continue,
            'x' => Token::X,
            '+' => Token::Operation(Operation::Add),
            '-' => Token::Operation(Operation::Sub),
            '*' => Token::Operation(Operation::Mul),
            '^' => Token::Operation(Operation::Pow),
            '/' => Token::Operation(Operation::Div),
            '(' => Token::BracketOpen,
            ')' => Token::BracketClosed,
            '0'..='9' => {
                let mut number = String::from(letter);
                while let Some('0'..='9' | '.') = function.get(0) {
                    number.push(function.remove(0));
                }
                Token::Number(number.parse().unwrap())
            }
            _ => Token::Unknown,
        };
        tokens.push(token);
    }
    tokens
}

fn parse_term(tokens: &mut Vec<Token>) -> Expression {
    let mut lhs = parse_factor(tokens);
    if lhs == Expression::Invalid { return Expression::Invalid }

    match tokens.get(0) {
        Some(Token::Operation(operation)) => {
            let operation = operation.clone();
            tokens.remove(0);
            let rhs = parse_term(tokens);
            if rhs == Expression::Invalid { return Expression::Invalid }
            lhs = Expression::Operation(Box::new(lhs), operation, Box::new(rhs))
        }
        _ => {},
    }

    lhs
}

fn parse_factor(tokens: &mut Vec<Token>) -> Expression {
    if tokens.is_empty() { return Expression::Invalid }
    let mut lhs = parse_single_value(tokens);
    if lhs == Expression::Invalid { return Expression::Invalid }

    loop {
        match tokens.get(0) {
            Some(Token::Operation(Operation::Mul | Operation::Div)) => {
                let operation = match tokens.remove(0) {
                    Token::Operation(operation) => operation,
                    _ => unreachable!()
                };
                let rhs = parse_factor(tokens);
                if rhs == Expression::Invalid { return Expression::Invalid }
                lhs = Expression::Operation(Box::new(lhs), operation, Box::new(rhs));
            },
            Some(Token::Operation(Operation::Pow)) => {
                tokens.remove(0);
                let rhs = parse_single_value(tokens);
                if rhs == Expression::Invalid { return Expression::Invalid }
                lhs = Expression::Operation(Box::new(lhs), Operation::Pow, Box::new(rhs));
            }
            _ => break,
        }
    }

    lhs
}

fn parse_single_value(tokens: &mut Vec<Token>) -> Expression {
    if tokens.is_empty() { return Expression::Invalid }
    match tokens.remove(0) {
        Token::X => Expression::X,
        Token::Number(val) => Expression::Number(val),
        Token::BracketOpen => {
            let lhs = parse_term(tokens);
            if let Some(Token::BracketClosed) = tokens.get(0) {  } else { return Expression::Invalid }
            tokens.remove(0);
            lhs
        }
        _ => Expression::Invalid,
    }
}

#[tauri::command]
fn y(x: Vec<f64>, expression: Expression) -> Vec<f64> {
    let mut y = vec![];
    for x in &x {
        y.push(expression.calc(*x));
    }
    y
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, parse, y])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
