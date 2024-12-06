use super::{calc_offset, identifier, space_delimited};
use crate::ast::{ExprEnum, Expression, Span};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, none_of},
    combinator::{cut, opt},
    multi::{fold_many0, many0},
    number::complete::recognize_float,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

fn factor(i: Span) -> IResult<Span, Expression> {
    alt((
        str_literal,
        num_literal,
        vec3_expr,
        func_call,
        ident,
        not_factor,
        parens,
    ))(i)
}

fn func_call(i: Span) -> IResult<Span, Expression> {
    let (r, ident) = space_delimited(identifier)(i)?;
    let (r, args) = space_delimited(delimited(
        tag("("),
        many0(delimited(multispace0, expr, space_delimited(opt(tag(","))))),
        tag(")"),
    ))(r)?;
    Ok((
        r,
        Expression {
            expr: ExprEnum::FnInvoke(ident, args),
            span: i,
        },
    ))
}

fn term(input: Span) -> IResult<Span, Expression> {
    let (r, init) = factor(input)?;

    let res = fold_many0(
        pair(space_delimited(alt((char('*'), char('/')))), factor),
        move || init.clone(),
        |acc, (op, val): (char, Expression)| {
            let span = calc_offset(input, acc.span);
            match op {
                '*' => Expression::new(ExprEnum::Mul(Box::new(acc), Box::new(val)), span),
                '/' => Expression::new(ExprEnum::Div(Box::new(acc), Box::new(val)), span),
                _ => panic!("Multiplicative expression should have '*' or '/' operator"),
            }
        },
    )(r);
    res
}

fn ident(input: Span) -> IResult<Span, Expression> {
    let (r, res) = space_delimited(identifier)(input)?;
    Ok((
        r,
        Expression {
            expr: ExprEnum::Ident(res),
            span: input,
        },
    ))
}

fn str_literal(i: Span) -> IResult<Span, Expression> {
    let (r0, _) = preceded(multispace0, char('\"'))(i)?;
    let (r, val) = many0(none_of("\""))(r0)?;
    let (r, _) = terminated(char('"'), multispace0)(r)?;
    Ok((
        r,
        Expression::new(
            ExprEnum::StrLiteral(
                val.iter()
                    .collect::<String>()
                    .replace("\\\\", "\\")
                    .replace("\\n", "\n"),
            ),
            i,
        ),
    ))
}

fn num_literal(input: Span) -> IResult<Span, Expression> {
    let (r, v) = space_delimited(recognize_float)(input)?;
    Ok((
        r,
        Expression::new(
            ExprEnum::NumLiteral(v.parse().map_err(|_| {
                nom::Err::Error(nom::error::Error {
                    input,
                    code: nom::error::ErrorKind::Digit,
                })
            })?),
            v,
        ),
    ))
}

fn parens(i: Span) -> IResult<Span, Expression> {
    space_delimited(delimited(tag("("), expr, tag(")")))(i)
}

fn not_factor(i: Span) -> IResult<Span, Expression> {
    let (i, _) = space_delimited(tag("!"))(i)?;
    let (i, cond) = cut(factor)(i)?;
    Ok((i, Expression::new(ExprEnum::Not(Box::new(cond)), i)))
}

fn num_expr(i: Span) -> IResult<Span, Expression> {
    let (r, init) = term(i)?;

    let res = fold_many0(
        pair(space_delimited(alt((char('+'), char('-')))), term),
        move || init.clone(),
        |acc, (op, val): (char, Expression)| {
            let span = calc_offset(i, acc.span);
            match op {
                '+' => Expression::new(ExprEnum::Add(Box::new(acc), Box::new(val)), span),
                '-' => Expression::new(ExprEnum::Sub(Box::new(acc), Box::new(val)), span),
                _ => panic!("Additive expression should have '+' or '-' operator"),
            }
        },
    )(r);
    res
}

fn cond_expr(i0: Span) -> IResult<Span, Expression> {
    let (i, first) = num_expr(i0)?;
    let (i, cond) = space_delimited(alt((
        tag("||"),
        tag("&&"),
        tag("<"),
        tag(">"),
        tag("=="),
        tag("!="),
    )))(i)?;
    let (i, second) = num_expr(i)?;
    let span = calc_offset(i0, i);
    Ok((
        i,
        match *cond.fragment() {
            "<" => Expression::new(ExprEnum::Lt(Box::new(first), Box::new(second)), span),
            ">" => Expression::new(ExprEnum::Gt(Box::new(first), Box::new(second)), span),
            "==" => Expression::new(ExprEnum::Eq(Box::new(first), Box::new(second)), span),
            "!=" => Expression::new(ExprEnum::Neq(Box::new(first), Box::new(second)), span),
            "&&" => Expression::new(ExprEnum::And(Box::new(first), Box::new(second)), span),
            "||" => Expression::new(ExprEnum::Or(Box::new(first), Box::new(second)), span),
            _ => unreachable!(),
        },
    ))
}

pub(super) fn expr(i: Span) -> IResult<Span, Expression> {
    alt((cond_expr, num_expr, vec3_expr))(i)
}

pub(super) fn vec3_expr(i0: Span) -> IResult<Span, Expression> {
    let (i, _) = space_delimited(tag("<"))(i0)?;
    let (i, x) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(","))(i)?;
    let (i, y) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(","))(i)?;
    let (i, z) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(">"))(i)?;
    Ok((
        i,
        Expression::new(
            ExprEnum::Vec3(Box::new(x), Box::new(y), Box::new(z)),
            calc_offset(i0, i),
        ),
    ))
}

pub(super) fn vec3_ident_expr(i: Span) -> IResult<Span, Expression> {
    alt((vec3_expr, ident))(i)
}

pub fn comment_expr(i: Span) -> IResult<Span, Expression> {
    let (i, _) = space_delimited(tag("//"))(i)?;
    let (i, _) = take_until("\n")(i)?;
    Ok((i, Expression::new(ExprEnum::NumLiteral(0.0), i)))
}
