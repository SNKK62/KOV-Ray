use super::{
    close_brace,
    expression::{expr, vec3_expr},
    material::material_expr,
    open_brace, space_delimited,
};
use crate::ast::{
    object::{AffineProperties, Rotate, RotateAxis},
    Expression, Object, Span,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::{many0, many1},
    sequence::delimited,
    IResult,
};

fn translate_decl(i: Span) -> IResult<Span, AffineProperties> {
    let (i, expr) = delimited(
        space_delimited(tag("translate:")),
        space_delimited(vec3_expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, AffineProperties::Translation(expr)))
}

fn rotate_x_decl(i: Span) -> IResult<Span, AffineProperties> {
    let (i, expr) = delimited(
        space_delimited(tag("rotateX:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((
        i,
        AffineProperties::Rotate(Rotate {
            axis: RotateAxis::X,
            expr,
        }),
    ))
}

fn rotate_y_decl(i: Span) -> IResult<Span, AffineProperties> {
    let (i, expr) = delimited(
        space_delimited(tag("rotateY:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((
        i,
        AffineProperties::Rotate(Rotate {
            axis: RotateAxis::Y,
            expr,
        }),
    ))
}

fn rotate_z_decl(i: Span) -> IResult<Span, AffineProperties> {
    let (i, expr) = delimited(
        space_delimited(tag("rotateZ:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((
        i,
        AffineProperties::Rotate(Rotate {
            axis: RotateAxis::Z,
            expr,
        }),
    ))
}

fn affine_properties(i: Span) -> IResult<Span, Vec<AffineProperties>> {
    let (i, res) = space_delimited(many0(alt((
        translate_decl,
        rotate_x_decl,
        rotate_y_decl,
        rotate_z_decl,
    ))))(i)?;

    Ok((i, res))
}

fn objects(i: Span) -> IResult<Span, Object> {
    let (i0, _) = space_delimited(tag("Objs"))(i)?;
    let (i, _) = space_delimited(open_brace)(i0)?;
    let (i, objects) = many1(object)(i)?;

    let mut affine: Vec<AffineProperties> = Vec::new();
    let mut i_start = i;
    let (i, p) = affine_properties(i_start)?;
    let (i, _) = space_delimited(close_brace)(i)?;
    affine.extend(p);
    i_start = i;
    Ok((i_start, Object::Objects { objects, affine }))
}

fn sphere_center_decl(i: Span) -> IResult<Span, (&str, Expression)> {
    let (i, expr) = delimited(
        space_delimited(tag("center:")),
        space_delimited(vec3_expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, ("center", expr)))
}

fn sphere_radius_decl(i: Span) -> IResult<Span, (&str, Expression)> {
    let (i, expr) = delimited(
        space_delimited(tag("radius:")),
        space_delimited(expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, ("radius", expr)))
}

fn material_decl(i: Span) -> IResult<Span, (&str, Expression)> {
    let (i, expr) = delimited(
        space_delimited(tag("material:")),
        space_delimited(material_expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, ("material", expr)))
}

fn sphere_object(i: Span) -> IResult<Span, Object> {
    let (i, _) = space_delimited(tag("Sphere"))(i)?;
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut center: Option<Expression> = None;
    let mut radius: Option<Expression> = None;
    let mut material: Option<Expression> = None;
    let mut affine: Vec<AffineProperties> = Vec::new();

    let (i, options) = many0(alt((sphere_center_decl, sphere_radius_decl, material_decl)))(i)?;

    options.iter().for_each(|(key, value)| match *key {
        "center" => center = Some(value.clone()),
        "radius" => radius = Some(value.clone()),
        "material" => material = Some(value.clone()),
        _ => {}
    });

    let (i, p) = affine_properties(i)?;
    let (i, _) = space_delimited(close_brace)(i)?;
    affine.extend(p);

    if center.is_none() || radius.is_none() || material.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    Ok((
        i,
        Object::Sphere {
            center: center.unwrap(),
            radius: radius.unwrap(),
            material: material.unwrap(),
            affine,
        },
    ))
}

#[derive(Debug)]
struct SquareObjectProperties<'src> {
    vertex: (Expression<'src>, Expression<'src>),
    material: Expression<'src>,
    affine: Vec<AffineProperties<'src>>,
}
#[derive(Debug)]
enum SquareObjectPropertiesEnum<'src> {
    Vertex((Expression<'src>, Expression<'src>)),
    Material(Expression<'src>),
}

fn vertex_decl(i: Span) -> IResult<Span, SquareObjectPropertiesEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("vertex:")),
        space_delimited(|i| {
            let (i, _) = tag("(")(i)?;
            let (i, v1) = vec3_expr(i)?;
            let (i, _) = tag(",")(i)?;
            let (i, v2) = vec3_expr(i)?;
            let (i, _) = tag(")")(i)?;
            Ok((i, (v1, v2)))
        }),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, SquareObjectPropertiesEnum::Vertex((expr.0, expr.1))))
}

fn square_material_decl(i: Span) -> IResult<Span, SquareObjectPropertiesEnum> {
    let (i, expr) = delimited(
        space_delimited(tag("material:")),
        space_delimited(material_expr),
        space_delimited(tag(",")),
    )(i)?;
    Ok((i, SquareObjectPropertiesEnum::Material(expr)))
}

fn general_square_object_properties(i: Span) -> IResult<Span, SquareObjectProperties> {
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut vertex: Option<(Expression, Expression)> = None;
    let mut material: Option<Expression> = None;
    let mut affine: Vec<AffineProperties> = Vec::new();

    let (i, options) = many0(alt((vertex_decl, square_material_decl)))(i)?;
    options.iter().for_each(|option| match option {
        SquareObjectPropertiesEnum::Vertex(v) => vertex = Some(v.clone()),
        SquareObjectPropertiesEnum::Material(m) => material = Some(m.clone()),
    });
    let (i, p) = affine_properties(i)?;
    affine.extend(p);

    let (i, _) = space_delimited(close_brace)(i)?;

    if vertex.is_none() || material.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i,
            code: nom::error::ErrorKind::Tag,
        }));
    }
    Ok((
        i,
        SquareObjectProperties {
            vertex: vertex.unwrap(),
            material: material.unwrap(),
            affine,
        },
    ))
}

fn box_object(i: Span) -> IResult<Span, Object> {
    let (i, _) = space_delimited(tag("Box"))(i)?;
    let (i, properties) = general_square_object_properties(i)?;
    Ok((
        i,
        Object::Box {
            vertex: properties.vertex,
            material: properties.material,
            affine: properties.affine,
        },
    ))
}

fn plane_object(i: Span) -> IResult<Span, Object> {
    let (i, _) = space_delimited(tag("Plane"))(i)?;
    let (i, properties) = general_square_object_properties(i)?;
    Ok((
        i,
        Object::Plane {
            vertex: properties.vertex,
            material: properties.material,
            affine: properties.affine,
        },
    ))
}

pub(super) fn object(i: Span) -> IResult<Span, Object> {
    alt((sphere_object, box_object, plane_object, objects))(i)
}
