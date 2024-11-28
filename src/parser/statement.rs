use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{cut, map_res, opt},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
    Finish, IResult,
};

use super::{
    calc_offset, close_brace,
    expression::{comment_expr, expr, vec3_ident_expr},
    identifier,
    object::object,
    open_brace, space_delimited,
};
use crate::ast::{CameraConfig, Config, Expression, Span, Statement, AST};

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
enum CameraConfigEnum<'a> {
    Lookfrom(Expression<'a>),
    Lookat(Expression<'a>),
    Up(Expression<'a>),
    Angle(Expression<'a>),
    DistToFocus(Expression<'a>),
}

fn loockfrom_decl(i: Span) -> IResult<Span, CameraConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("lookfrom:")),
        space_delimited(vec3_ident_expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, CameraConfigEnum::Lookfrom(expr)))
}

fn loockat_decl(i: Span) -> IResult<Span, CameraConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("lookat:")),
        space_delimited(vec3_ident_expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, CameraConfigEnum::Lookat(expr)))
}

fn up_decl(i: Span) -> IResult<Span, CameraConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("up:")),
        space_delimited(vec3_ident_expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, CameraConfigEnum::Up(expr)))
}

fn angle_decl(i: Span) -> IResult<Span, CameraConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("angle:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, CameraConfigEnum::Angle(expr)))
}

fn dist_to_focus_decl(i: Span) -> IResult<Span, CameraConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("dist_to_focus:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, CameraConfigEnum::DistToFocus(expr)))
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

    let (i, p) = many0(alt((
        loockfrom_decl,
        loockat_decl,
        up_decl,
        angle_decl,
        dist_to_focus_decl,
    )))(i)?;

    p.iter().for_each(|v| match v {
        CameraConfigEnum::Lookfrom(expr) => lookfrom = Some(expr.clone()),
        CameraConfigEnum::Lookat(expr) => lookat = Some(expr.clone()),
        CameraConfigEnum::Up(expr) => up = Some(expr.clone()),
        CameraConfigEnum::Angle(expr) => angle = Some(expr.clone()),
        CameraConfigEnum::DistToFocus(expr) => dist_to_focus = Some(expr.clone()),
    });

    if lookfrom.is_none() || lookat.is_none() || angle.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    let (i, _) = space_delimited(close_brace)(i)?;

    Ok((
        i,
        (Statement::Camera {
            span: calc_offset(i0, i),
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

#[derive(Debug)]
enum ConfigEnum<'a> {
    Width(Expression<'a>),
    Height(Expression<'a>),
    SamplesPerPixel(Expression<'a>),
    MaxDepth(Expression<'a>),
    SkyColor(Expression<'a>),
}

fn width_decl(i: Span) -> IResult<Span, ConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("width:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, ConfigEnum::Width(expr)))
}

fn height_decl(i: Span) -> IResult<Span, ConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("height:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, ConfigEnum::Height(expr)))
}

fn samples_per_pixel_decl(i: Span) -> IResult<Span, ConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("samples_per_pixel:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, ConfigEnum::SamplesPerPixel(expr)))
}

fn max_depth_decl(i: Span) -> IResult<Span, ConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("max_depth:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, ConfigEnum::MaxDepth(expr)))
}

fn sky_color_decl(i: Span) -> IResult<Span, ConfigEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("sky_color:")),
        space_delimited(vec3_ident_expr),
        space_delimited(tag(",")),
    )(i)?;
    let (i, _) = opt(space_delimited(comment_expr))(i)?;
    Ok((i, ConfigEnum::SkyColor(expr)))
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

    let (i, p) = many0(alt((
        width_decl,
        height_decl,
        samples_per_pixel_decl,
        max_depth_decl,
        sky_color_decl,
    )))(i)?;

    p.iter().for_each(|v| match v {
        ConfigEnum::Width(expr) => width = Some(expr.clone()),
        ConfigEnum::Height(expr) => height = Some(expr.clone()),
        ConfigEnum::SamplesPerPixel(expr) => samples_per_pixel = Some(expr.clone()),
        ConfigEnum::MaxDepth(expr) => max_depth = Some(expr.clone()),
        ConfigEnum::SkyColor(expr) => sky_color = Some(expr.clone()),
    });

    if samples_per_pixel.is_none() || width.is_none() || height.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    let (i, _) = space_delimited(close_brace)(i)?;

    Ok((
        i,
        (Statement::Config {
            span: calc_offset(i0, i),
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
    let (i, ex) = space_delimited(comment_expr)(i)?;
    Ok((i, Statement::Expression(ex)))
}

pub fn statement(i: Span) -> IResult<Span, Statement> {
    alt((
        object_statement,
        camera_statement,
        config_statement,
        var_assign,
        if_statement,
        while_statement,
        terminated(break_statement, pair(tag(";"), multispace0)),
        terminated(continue_statement, pair(tag(";"), multispace0)),
        terminated(expr_statement, pair(tag(";"), multispace0)),
        comment_statement,
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
