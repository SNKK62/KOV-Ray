use super::{
    calc_offset,
    expression::{expr, vec3_ident_expr},
    space_delimited,
    texture::texture_expr,
};
use crate::ast::{ExprEnum, Expression, Material, Span};
use nom::{branch::alt, bytes::complete::tag, IResult};

fn metal_material(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Metal"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, color) = space_delimited(vec3_ident_expr)(i)?;
    let (i, _) = space_delimited(tag(","))(i)?;
    let (i, fuzz) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Material(Box::new(Material::Metal { color, fuzz })),
        },
    ))
}

fn dielectric_material(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Dielectric"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, reflection_index) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Material(Box::new(Material::Dielectric { reflection_index })),
        },
    ))
}

fn lambertian_material(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Lambertian"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, texture) = space_delimited(texture_expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Material(Box::new(Material::Lambertian { texture })),
        },
    ))
}

fn light_material(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Light"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, color) = space_delimited(vec3_ident_expr)(i)?;
    let (i, _) = space_delimited(tag(","))(i)?;
    let (i, intensity) = space_delimited(expr)(i)?;
    let (i, _) = space_delimited(tag(")"))(i)?;
    Ok((
        i,
        Expression {
            span: calc_offset(i0, i),
            expr: ExprEnum::Material(Box::new(Material::Light { color, intensity })),
        },
    ))
}

pub(super) fn material_expr(i: Span) -> IResult<Span, Expression> {
    alt((
        metal_material,
        dielectric_material,
        lambertian_material,
        light_material,
    ))(i)
}
