//! Parametric expressions: `Expr` AST, evaluation against a [`VariableTable`]
//! and a small recursive-descent parser for formulas such as
//! `(width - 2 * gap) / 2` or `max(a, b)`.

use crate::error::{CadError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A parametric expression node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Const(f64),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Min(Box<Expr>, Box<Expr>),
    Max(Box<Expr>, Box<Expr>),
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        otherwise: Box<Expr>,
    },
}

impl Expr {
    /// Constant expression constructor.
    pub fn constant(v: f64) -> Self {
        Expr::Const(v)
    }

    /// Variable expression constructor.
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    /// Evaluate the expression against the given variable table.
    pub fn eval(&self, vars: &VariableTable) -> Result<f64> {
        match self {
            Expr::Const(v) => Ok(*v),
            Expr::Var(n) => vars
                .get(n)
                .ok_or_else(|| CadError::UndefinedVariable(n.clone())),
            Expr::Add(a, b) => Ok(a.eval(vars)? + b.eval(vars)?),
            Expr::Sub(a, b) => Ok(a.eval(vars)? - b.eval(vars)?),
            Expr::Mul(a, b) => Ok(a.eval(vars)? * b.eval(vars)?),
            Expr::Div(a, b) => {
                let d = b.eval(vars)?;
                if d == 0.0 {
                    Err(CadError::DivisionByZero)
                } else {
                    Ok(a.eval(vars)? / d)
                }
            }
            Expr::Neg(a) => Ok(-a.eval(vars)?),
            Expr::Min(a, b) => Ok(a.eval(vars)?.min(b.eval(vars)?)),
            Expr::Max(a, b) => Ok(a.eval(vars)?.max(b.eval(vars)?)),
            Expr::If {
                cond,
                then,
                otherwise,
            } => {
                if cond.eval(vars)? != 0.0 {
                    then.eval(vars)
                } else {
                    otherwise.eval(vars)
                }
            }
        }
    }

    /// Names of all variables referenced by this expression.
    pub fn variables(&self, out: &mut Vec<String>) {
        match self {
            Expr::Const(_) => {}
            Expr::Var(n) => out.push(n.clone()),
            Expr::Add(a, b)
            | Expr::Sub(a, b)
            | Expr::Mul(a, b)
            | Expr::Div(a, b)
            | Expr::Min(a, b)
            | Expr::Max(a, b) => {
                a.variables(out);
                b.variables(out);
            }
            Expr::Neg(a) => a.variables(out),
            Expr::If {
                cond,
                then,
                otherwise,
            } => {
                cond.variables(out);
                then.variables(out);
                otherwise.variables(out);
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(v) => write!(f, "{v}"),
            Expr::Var(n) => write!(f, "{n}"),
            Expr::Add(a, b) => write!(f, "({a} + {b})"),
            Expr::Sub(a, b) => write!(f, "({a} - {b})"),
            Expr::Mul(a, b) => write!(f, "({a} * {b})"),
            Expr::Div(a, b) => write!(f, "({a} / {b})"),
            Expr::Neg(a) => write!(f, "(-{a})"),
            Expr::Min(a, b) => write!(f, "min({a}, {b})"),
            Expr::Max(a, b) => write!(f, "max({a}, {b})"),
            Expr::If {
                cond,
                then,
                otherwise,
            } => write!(f, "if({cond}, {then}, {otherwise})"),
        }
    }
}

/// Table of named numeric variables used by [`Expr::eval`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariableTable {
    map: HashMap<String, f64>,
}

impl VariableTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert/overwrite a variable value.
    pub fn set(&mut self, name: impl Into<String>, value: f64) {
        self.map.insert(name.into(), value);
    }

    /// Get a variable value.
    pub fn get(&self, name: &str) -> Option<f64> {
        self.map.get(name).copied()
    }

    /// Remove a variable, returning its previous value.
    pub fn remove(&mut self, name: &str) -> Option<f64> {
        self.map.remove(name)
    }

    /// Iterate over all names (sorted for deterministic UI display).
    pub fn names(&self) -> Vec<&String> {
        let mut v: Vec<&String> = self.map.keys().collect();
        v.sort();
        v
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a formula string into an [`Expr`].
///
/// Grammar (precedence low → high):
/// ```text
/// expr   := term (('+'|'-') term)*
/// term   := atom (('*'|'/') atom)*
/// atom   := number | ident ['(' args ')'] | '-' atom | '(' expr ')'
/// ```
pub fn parse_expr(input: &str) -> Result<Expr> {
    let mut p = Parser {
        s: input.as_bytes(),
        pos: 0,
    };
    p.skip_ws();
    let e = p.expr()?;
    p.skip_ws();
    if p.pos < p.s.len() {
        return Err(CadError::other(format!(
            "unexpected character `{}` at position {}",
            (p.s[p.pos] as char),
            p.pos
        )));
    }
    Ok(e)
}

struct Parser<'a> {
    s: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expr(&mut self) -> Result<Expr> {
        let mut lhs = self.term()?;
        loop {
            self.skip_ws();
            if self.eat(b'+') {
                let rhs = self.term()?;
                lhs = Expr::Add(Box::new(lhs), Box::new(rhs));
            } else if self.eat(b'-') {
                let rhs = self.term()?;
                lhs = Expr::Sub(Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut lhs = self.atom()?;
        loop {
            self.skip_ws();
            if self.eat(b'*') {
                let rhs = self.atom()?;
                lhs = Expr::Mul(Box::new(lhs), Box::new(rhs));
            } else if self.eat(b'/') {
                let rhs = self.atom()?;
                lhs = Expr::Div(Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn atom(&mut self) -> Result<Expr> {
        self.skip_ws();
        match self.peek() {
            None => Err(CadError::other("unexpected end of expression")),
            Some(b'(') => {
                self.pos += 1;
                let e = self.expr()?;
                if !self.eat(b')') {
                    return Err(CadError::other("expected `)`"));
                }
                Ok(e)
            }
            Some(b'-') => {
                self.pos += 1;
                let inner = self.atom()?;
                Ok(Expr::Neg(Box::new(inner)))
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.number(),
            Some(c) if c.is_ascii_alphabetic() || c == b'_' => self.ident_or_call(),
            Some(c) => Err(CadError::other(format!(
                "unexpected character `{}`",
                c as char
            ))),
        }
    }

    fn number(&mut self) -> Result<Expr> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.s[start..self.pos])
            .map_err(|_| CadError::other("invalid number literal"))?;
        let v: f64 = text
            .parse()
            .map_err(|_| CadError::other(format!("bad number `{text}`")))?;
        Ok(Expr::Const(v))
    }

    fn ident_or_call(&mut self) -> Result<Expr> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let name = std::str::from_utf8(&self.s[start..self.pos]).unwrap().to_string();
        self.skip_ws();
        if self.peek() == Some(b'(') {
            // function call
            self.pos += 1;
            let mut args = vec![self.expr()?];
            while self.eat(b',') {
                args.push(self.expr()?);
            }
            if !self.eat(b')') {
                return Err(CadError::other(format!("expected `)` after {name}(...)")));
            }
            match (name.as_str(), args.len()) {
                ("min", 2) => Ok(Expr::Min(Box::new(args[0].clone()), Box::new(args[1].clone()))),
                ("max", 2) => Ok(Expr::Max(Box::new(args[0].clone()), Box::new(args[1].clone()))),
                ("if", 3) => Ok(Expr::If {
                    cond: Box::new(args[0].clone()),
                    then: Box::new(args[1].clone()),
                    otherwise: Box::new(args[2].clone()),
                }),
                (n, _) => Err(CadError::other(format!("unknown function `{n}`"))),
            }
        } else {
            Ok(Expr::Var(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> VariableTable {
        let mut v = VariableTable::new();
        v.set("width", 600.0);
        v.set("height", 720.0);
        v.set("depth", 560.0);
        v.set("panel_thickness", 18.0);
        v.set("gap", 3.0);
        v
    }

    #[test]
    fn eval_simple() {
        let e = parse_expr("(width - 2 * gap) / 2").unwrap();
        assert!((e.eval(&vars()).unwrap() - 297.0).abs() < 1e-9);
    }

    #[test]
    fn eval_functions() {
        let e = parse_expr("min(width, height)").unwrap();
        assert_eq!(e.eval(&vars()).unwrap(), 600.0);
        let e = parse_expr("max(width, height)").unwrap();
        assert_eq!(e.eval(&vars()).unwrap(), 720.0);
        let e = parse_expr("if(gap, 1, 2)").unwrap();
        assert_eq!(e.eval(&vars()).unwrap(), 1.0);
    }

    #[test]
    fn unary_minus_and_precedence() {
        let e = parse_expr("-gap + width * 2").unwrap();
        assert!((e.eval(&vars()).unwrap() - (1200.0 - 3.0)).abs() < 1e-9);
    }

    #[test]
    fn undefined_variable_errors() {
        let e = parse_expr("foo + 1").unwrap();
        assert!(matches!(
            e.eval(&vars()),
            Err(CadError::UndefinedVariable(n)) if n == "foo"
        ));
    }

    #[test]
    fn division_by_zero_errors() {
        let e = parse_expr("width / 0").unwrap();
        assert!(matches!(e.eval(&vars()), Err(CadError::DivisionByZero)));
    }

    #[test]
    fn parse_errors() {
        assert!(parse_expr("1 +").is_err());
        assert!(parse_expr("(1 + 2").is_err());
        assert!(parse_expr("@").is_err());
        assert!(parse_expr("sqrt(2)").is_err());
    }

    #[test]
    fn variables_collected() {
        let e = parse_expr("width + gap * 2").unwrap();
        let mut v = Vec::new();
        e.variables(&mut v);
        v.sort();
        assert_eq!(v, vec!["gap".to_string(), "width".to_string()]);
    }

    #[test]
    fn display_roundtrip() {
        let e = parse_expr("width - 2 * gap").unwrap();
        let text = e.to_string();
        let again = parse_expr(&text).unwrap();
        assert_eq!(e, again);
    }
}
