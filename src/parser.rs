use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::recognize,
    error::ParseError,
    multi::many0,
    sequence::{delimited, pair},
    IResult, InputTake, Offset, Parser,
};
use std::{collections::HashMap, error::Error};

use crate::ast::{Span, AST};
mod expression;
mod material;
mod object;
mod statement;
mod texture;
use statement::statements_finish;

pub type Functions<'src> = HashMap<String, FnDecl>;

fn unary_fn(f: fn(f64) -> f64) -> FnDecl {
    FnDecl::Native(NativeFn {
        code: Box::new(move |args| {
            let arg = args.iter().next().expect("function missing argument");
            f(*arg)
        }),
    })
}

fn binary_fn(f: fn(f64, f64) -> f64) -> FnDecl {
    FnDecl::Native(NativeFn {
        code: Box::new(move |args| {
            let mut args = args.iter();
            let lhs = args.next().expect("function missing argument");
            let rhs = args.next().expect("function missing argument");
            f(*lhs, *rhs)
        }),
    })
}

pub fn standard_functions<'src>() -> Functions<'src> {
    let mut funcs = Functions::new();
    funcs.insert("sqrt".to_string(), unary_fn(f64::sqrt));
    funcs.insert("sin".to_string(), unary_fn(f64::sin));
    funcs.insert("cos".to_string(), unary_fn(f64::cos));
    funcs.insert("tan".to_string(), unary_fn(f64::tan));
    funcs.insert("asin".to_string(), unary_fn(f64::asin));
    funcs.insert("acos".to_string(), unary_fn(f64::acos));
    funcs.insert("atan".to_string(), unary_fn(f64::atan));
    funcs.insert("atan2".to_string(), binary_fn(f64::atan2));
    funcs.insert("pow".to_string(), binary_fn(f64::powf));
    funcs.insert("exp".to_string(), unary_fn(f64::exp));
    funcs.insert("log".to_string(), binary_fn(f64::log));
    funcs.insert("log10".to_string(), unary_fn(f64::log10));
    funcs
}

pub enum FnDecl {
    Native(NativeFn),
}

type NativeFnCode = dyn Fn(&[f64]) -> f64;
pub struct NativeFn {
    pub code: Box<NativeFnCode>,
}

fn space_delimited<'src, O, E>(
    f: impl Parser<Span<'src>, O, E>,
) -> impl FnMut(Span<'src>) -> IResult<Span<'src>, O, E>
where
    E: ParseError<Span<'src>>,
{
    delimited(multispace0, f, multispace0)
}

fn open_brace(i: Span) -> IResult<Span, ()> {
    let (i, _) = space_delimited(char('{'))(i)?;
    Ok((i, ()))
}

fn close_brace(i: Span) -> IResult<Span, ()> {
    let (i, _) = space_delimited(char('}'))(i)?;
    Ok((i, ()))
}

fn identifier(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn calc_offset<'a>(i: Span<'a>, r: Span<'a>) -> Span<'a> {
    i.take(i.offset(&r))
}

pub fn parse<'a>(i: &'a str) -> Result<AST<'a>, Box<dyn Error + 'a>> {
    let i = Span::new(i);
    let res = statements_finish(i)?;
    Ok(res)
}
