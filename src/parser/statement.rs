use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::{cut, map_res, opt},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
    Finish, IResult,
};

use super::{
    calc_offset, close_brace,
    expression::{expr, vec3_expr, vec3_ident_expr},
    identifier,
    object::object,
    open_brace, space_delimited,
};
use crate::ast::{CameraConfig, Config, ExprEnum, Expression, Span, Statement, AST};

fn object_statement(i0: Span) -> IResult<Span, Statement> {
    let (i, object) = object(i0)?;
    Ok((
        i,
        Statement::Object {
            span: calc_offset(i0, i),
            object,
        },
    ))
}

fn camera_statement(i: Span) -> IResult<Span, Statement> {
    let (i, _) = space_delimited(tag("Camera"))(i)?;
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut lookfrom: Option<Expression> = None;
    let mut lookat: Option<Expression> = None;
    let mut up: Option<Expression> = None;
    let mut angle: Option<Expression> = None;
    let mut dist_to_focus: Option<Expression> = None;
    let i0 = i;
    let mut i_start = i;
    loop {
        let mut is_updated = false;
        let (i, attr) = opt(preceded(
            space_delimited(tag("lookfrom:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            is_updated = true;
            lookfrom = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("lookat:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            lookat = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("up:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            up = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("angle:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            angle = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("dist_to_focus:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            dist_to_focus = Some(attr.0);
        }
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
        if !is_updated {
            return Err(nom::Err::Error(nom::error::Error {
                input: i_start,
                code: nom::error::ErrorKind::Tag,
            }));
        }
    }

    if lookfrom.is_none() || lookat.is_none() || angle.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i_start,
            code: nom::error::ErrorKind::Tag,
        }));
    }
    Ok((
        i_start,
        (Statement::Camera {
            span: calc_offset(i0, i_start),
            config: CameraConfig {
                lookfrom: lookfrom.unwrap(),
                lookat: lookat.unwrap(),
                up,
                angle: angle.unwrap(),
                dist_to_focus,
            },
        }),
    ))
}

fn config_statement(i: Span) -> IResult<Span, Statement> {
    let (i, _) = space_delimited(tag("Config"))(i)?;
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut width: Option<Expression> = None;
    let mut height: Option<Expression> = None;
    let mut samples_per_pixel: Option<Expression> = None;
    let mut max_depth: Option<Expression> = None;
    let mut sky_color: Option<Expression> = None;
    let i0 = i;
    let mut i_start = i;
    loop {
        let mut is_updated = false;
        let (i, attr) = opt(preceded(
            space_delimited(tag("width:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            is_updated = true;
            width = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("height:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            height = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("samples_per_pixel:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            samples_per_pixel = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("max_depth:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            max_depth = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("sky_color:")),
            pair(space_delimited(vec3_ident_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            is_updated = true;
            sky_color = Some(attr.0);
        }
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
        if !is_updated {
            return Err(nom::Err::Error(nom::error::Error {
                input: i_start,
                code: nom::error::ErrorKind::Tag,
            }));
        }
    }

    if samples_per_pixel.is_none() || width.is_none() || height.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i_start,
            code: nom::error::ErrorKind::Tag,
        }));
    }
    Ok((
        i_start,
        (Statement::Config {
            span: calc_offset(i0, i_start),
            config: Config {
                width: width.unwrap(),
                height: height.unwrap(),
                samples_per_pixel: samples_per_pixel.unwrap(),
                max_depth,
                sky_color,
            },
        }),
    ))
}

fn var_assign(i: Span) -> IResult<Span, Statement> {
    let span = i;
    let (i, name) = space_delimited(identifier)(i)?;
    let (i, _) = space_delimited(char('='))(i)?;
    let (i, ex) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(char(';'))(i)?;
    Ok((
        i,
        Statement::VarAssign {
            span: calc_offset(span, i),
            name,
            ex,
        },
    ))
}

fn expr_statement(i: Span) -> IResult<Span, Statement> {
    let (i, res) = expr(i)?;
    Ok((i, Statement::Expression(res)))
}

fn if_statement(i: Span) -> IResult<Span, Statement> {
    let (i0, _) = space_delimited(tag("if"))(i)?;
    let (i, cond) = expr(i0)?;
    let (i, t_case) = delimited(open_brace, statements, close_brace)(i)?;
    let (i, f_case) = opt(preceded(
        space_delimited(tag("else")),
        alt((
            delimited(open_brace, statements, close_brace),
            map_res(
                if_statement,
                |v| -> Result<Vec<Statement>, nom::error::Error<&str>> { Ok(vec![v]) },
            ),
        )),
    ))(i)?;

    Ok((
        i,
        Statement::If {
            cond: Box::new(cond),
            stmts: Box::new(t_case),
            else_stmts: f_case.map(Box::new),
            span: calc_offset(i0, i),
        },
    ))
}

fn while_statement(i: Span) -> IResult<Span, Statement> {
    let i0 = i;
    let (i, _) = space_delimited(tag("while"))(i)?;
    let (i, (cond, stmts)) = cut(|i| {
        let (i, cond) = space_delimited(expr)(i)?;
        let (i, stmts) = delimited(open_brace, statements, close_brace)(i)?;
        Ok((i, (cond, stmts)))
    })(i)?;
    Ok((
        i,
        Statement::While {
            span: calc_offset(i0, i),
            cond,
            stmts,
        },
    ))
}

fn break_statement(i: Span) -> IResult<Span, Statement> {
    let (i, _) = space_delimited(tag("break"))(i)?;
    Ok((i, Statement::Break))
}

fn continue_statement(i: Span) -> IResult<Span, Statement> {
    let (i, _) = space_delimited(tag("continue"))(i)?;
    Ok((i, Statement::Continue))
}

fn comment_statement(i: Span) -> IResult<Span, Statement> {
    let (i, _) = space_delimited(tag("//"))(i)?;
    let (i, _) = take_until("\n")(i)?;
    Ok((
        i,
        Statement::Expression(Expression::new(ExprEnum::NumLiteral(0.0), i)),
    ))
}

pub fn statement(i: Span) -> IResult<Span, Statement> {
    alt((
        object_statement,
        camera_statement,
        config_statement,
        comment_statement,
        var_assign,
        if_statement,
        while_statement,
        terminated(break_statement, pair(tag(";"), multispace0)),
        terminated(continue_statement, pair(tag(";"), multispace0)),
        terminated(expr_statement, pair(tag(";"), multispace0)),
    ))(i)
}

fn statements(i: Span) -> IResult<Span, AST> {
    let (i, stmts) = many0(statement)(i)?;
    let (i, _) = opt(multispace0)(i)?;
    Ok((i, stmts))
}

pub fn statements_finish(i: Span) -> Result<AST, nom::error::Error<Span>> {
    let (_, res) = statements(i).finish()?;
    Ok(res)
}
