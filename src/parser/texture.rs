use super::{
    calc_offset,
    expression::{expr, vec3_expr},
    space_delimited,
};
use crate::ast::{ExprEnum, Expression, Span, Texture};
use nom::{branch::alt, bytes::complete::tag, IResult};

fn solid_texture(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Solid"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, color) = space_delimited(vec3_expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Texture(Box::new(Texture::SolidColor(color))),
        },
    ))
}

fn checker_texture(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Checker"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, odd) = space_delimited(vec3_expr)(i)?;
    let (i, _) = space_delimited(tag(","))(i)?;
    let (i, even) = space_delimited(vec3_expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Texture(Box::new(Texture::Checker(odd, even))),
        },
    ))
}

fn perlin_texture(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Perlin"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, scale) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Texture(Box::new(Texture::Perlin(scale))),
        },
    ))
}

pub(super) fn texture_expr(i: Span) -> IResult<Span, Expression> {
    // TODO: ImageTexture
    alt((perlin_texture, checker_texture, solid_texture))(i)
}
