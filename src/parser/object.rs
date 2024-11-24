use super::{
    close_brace,
    expression::{expr, vec3_expr},
    material::material_expr,
    open_brace, space_delimited,
};
use crate::ast::{object::AffineProperties, Expression, Object, Span};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::opt,
    multi::many1,
    sequence::{pair, preceded, tuple},
    IResult,
};
use std::{cell::RefCell, rc::Rc};

fn affine_properties<'a>(
    i: Span<'a>,
    properties: &Rc<RefCell<AffineProperties<'a>>>,
    // properties: &mut std::cell::RefMut<AffineProperties<'a>>,
) -> IResult<Span<'a>, ()> {
    let (i, attr) = opt(preceded(
        space_delimited(tag("translate:")),
        pair(space_delimited(vec3_expr), space_delimited(tag(","))),
    ))(i)?;
    if let Some(attr) = attr {
        properties.borrow_mut().translate = Some(attr.0);
    }
    let (i, attr) = opt(preceded(
        space_delimited(tag("rotateX:")),
        pair(space_delimited(expr), space_delimited(tag(","))),
    ))(i)?;
    if let Some(attr) = attr {
        properties.borrow_mut().push_rotate(0, attr.0);
    }
    let (i, attr) = opt(preceded(
        space_delimited(tag("rotateY:")),
        pair(space_delimited(expr), space_delimited(tag(","))),
    ))(i)?;
    if let Some(attr) = attr {
        properties.borrow_mut().push_rotate(1, attr.0);
    }
    let (i, attr) = opt(preceded(
        space_delimited(tag("rotateZ:")),
        pair(space_delimited(expr), space_delimited(tag(","))),
    ))(i)?;
    if let Some(attr) = attr {
        properties.borrow_mut().push_rotate(2, attr.0);
    }
    Ok((i, ()))
}

fn objects(i: Span) -> IResult<Span, Object> {
    let (i0, _) = space_delimited(tag("Objs"))(i)?;
    let (i, _) = space_delimited(open_brace)(i0)?;
    let (i, objects) = many1(object)(i)?;

    let affine = Rc::new(RefCell::new(AffineProperties::new()));
    let mut i_start = i;
    loop {
        let (i, _) = affine_properties(i_start, &affine)?;
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
    }
    let affine = affine.borrow().clone();
    Ok((i_start, Object::Objects { objects, affine }))
}

fn sphere_object(i: Span) -> IResult<Span, Object> {
    let (i, _) = space_delimited(tag("Sphere"))(i)?;
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut center: Option<Expression> = None;
    let mut radius: Option<Expression> = None;
    let mut material: Option<Expression> = None;
    let affine = Rc::new(RefCell::new(AffineProperties::new()));

    let mut i_start = i;
    loop {
        let (i, attr) = opt(preceded(
            space_delimited(tag("center:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            center = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("radius:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            radius = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("material:")),
            pair(space_delimited(material_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            material = Some(attr.0);
        }
        let (i, _) = affine_properties(i, &affine)?;
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
    }

    if center.is_none() || radius.is_none() || material.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i_start,
            code: nom::error::ErrorKind::Tag,
        }));
    }
    let affine = affine.borrow().clone();

    Ok((
        i_start,
        Object::Sphere {
            center: center.unwrap(),
            radius: radius.unwrap(),
            material: material.unwrap(),
            affine,
        },
    ))
}

struct SquareObjectProperties<'src> {
    vertex: (Expression<'src>, Expression<'src>),
    material: Expression<'src>,
    affine: AffineProperties<'src>,
}
fn general_square_object_properties(i: Span) -> IResult<Span, SquareObjectProperties> {
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut vertex: Option<(Expression, Expression)> = None;
    let mut material: Option<Expression> = None;
    let affine = Rc::new(RefCell::new(AffineProperties::new()));

    let mut i_start = i;
    loop {
        let (i, attr) = opt(preceded(
            space_delimited(tag("vertex:")),
            tuple((
                space_delimited(tag("(")),
                space_delimited(vec3_expr),
                space_delimited(tag(",")),
                space_delimited(vec3_expr),
                space_delimited(tag(")")),
                space_delimited(tag(",")),
            )),
        ))(i_start)?;
        if let Some(attr) = attr {
            vertex = Some((attr.1, attr.3));
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("material:")),
            pair(space_delimited(material_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            material = Some(attr.0);
        }
        let (i, _) = affine_properties(i, &affine)?;
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
    }

    if vertex.is_none() || material.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i_start,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    let affine = affine.borrow().clone();
    Ok((
        i_start,
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
