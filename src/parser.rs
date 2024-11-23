use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, char, multispace0, none_of},
    combinator::{cut, map_res, opt, recognize},
    error::ParseError,
    multi::{fold_many0, many0, many1},
    number::complete::recognize_float,
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, InputTake, Offset, Parser,
};
use std::{collections::HashMap, error::Error};

use crate::ast::{ExprEnum, Expression, Material, Object, Span, Statement, Texture, AST};

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

fn calc_offset<'a>(i: Span<'a>, r: Span<'a>) -> Span<'a> {
    i.take(i.offset(&r))
}

fn factor(i: Span) -> IResult<Span, Expression> {
    alt((
        str_literal,
        num_literal,
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

fn identifier(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
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

fn open_brace(i: Span) -> IResult<Span, ()> {
    let (i, _) = space_delimited(char('{'))(i)?;
    Ok((i, ()))
}

fn close_brace(i: Span) -> IResult<Span, ()> {
    let (i, _) = space_delimited(char('}'))(i)?;
    Ok((i, ()))
}

pub fn expr(i: Span) -> IResult<Span, Expression> {
    alt((cond_expr, num_expr))(i)
}

fn vec3_expr(i0: Span) -> IResult<Span, Expression> {
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

fn metal_material(i: Span) -> IResult<Span, Expression> {
    let (i0, _) = space_delimited(tag("Metal"))(i)?;
    let (i, _) = space_delimited(tag("("))(i0)?;
    let (i, color) = space_delimited(vec3_expr)(i)?;
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
    let (i, texture) = space_delimited(texture)(i)?;
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
    let (i, color) = space_delimited(vec3_expr)(i)?;
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

fn material_expr(i: Span) -> IResult<Span, Expression> {
    alt((
        metal_material,
        dielectric_material,
        lambertian_material,
        light_material,
    ))(i)
}

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

fn texture(i: Span) -> IResult<Span, Expression> {
    // TODO: ImageTexture
    alt((perlin_texture, checker_texture, solid_texture))(i)
}

fn objects(i: Span) -> IResult<Span, Object> {
    let (i0, _) = space_delimited(tag("Objs"))(i)?;
    let (i, _) = space_delimited(open_brace)(i0)?;
    let (i, objects) = many1(object)(i)?;

    let mut translate: Option<Expression> = None;
    let mut rotate: Option<Expression> = None;

    let mut i_start = i;
    loop {
        let (i, attr) = opt(preceded(
            space_delimited(tag("translate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            translate = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("rotate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            rotate = Some(attr.0);
        }
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
    }
    Ok((
        i_start,
        Object::Objects {
            objects,
            translate,
            rotate,
        },
    ))
}

fn sphere_object(i: Span) -> IResult<Span, Object> {
    let (i, _) = space_delimited(tag("Sphere"))(i)?;
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut center: Option<Expression> = None;
    let mut radius: Option<Expression> = None;
    let mut translate: Option<Expression> = None;
    let mut rotate: Option<Expression> = None;
    let mut material: Option<Expression> = None;

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
        let (i, attr) = opt(preceded(
            space_delimited(tag("translate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            translate = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("rotate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            rotate = Some(attr.0);
        }
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

    Ok((
        i_start,
        Object::Sphere {
            center: center.unwrap(),
            radius: radius.unwrap(),
            material: material.unwrap(),
            translate,
            rotate,
        },
    ))
}

struct SquareObjectProperties<'src> {
    vertex: (Expression<'src>, Expression<'src>),
    material: Expression<'src>,
    translate: Option<Expression<'src>>,
    rotate: Option<Expression<'src>>,
}
fn general_square_object_properties(i: Span) -> IResult<Span, SquareObjectProperties> {
    let (i, _) = space_delimited(open_brace)(i)?;

    let mut vertex: Option<(Expression, Expression)> = None;
    let mut translate: Option<Expression> = None;
    let mut rotate: Option<Expression> = None;
    let mut material: Option<Expression> = None;

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
        println!("attr:{:?}", attr);
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
        let (i, attr) = opt(preceded(
            space_delimited(tag("translate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            translate = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("rotate:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            rotate = Some(attr.0);
        }
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

    Ok((
        i_start,
        SquareObjectProperties {
            vertex: vertex.unwrap(),
            material: material.unwrap(),
            translate,
            rotate,
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
            translate: properties.translate,
            rotate: properties.rotate,
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
            translate: properties.translate,
            rotate: properties.rotate,
        },
    ))
}

fn object(i: Span) -> IResult<Span, Object> {
    alt((sphere_object, box_object, plane_object, objects))(i)
}

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
    let mut angle: Option<Expression> = None;
    let i0 = i;
    let mut i_start = i;
    loop {
        let (i, attr) = opt(preceded(
            space_delimited(tag("lookfrom:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            lookfrom = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("lookat:")),
            pair(space_delimited(vec3_expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            lookat = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("angle:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            angle = Some(attr.0);
        }
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
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
            lookfrom: lookfrom.unwrap(),
            lookat: lookat.unwrap(),
            angle: angle.unwrap(),
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
    let i0 = i;
    let mut i_start = i;
    loop {
        let (i, attr) = opt(preceded(
            space_delimited(tag("width:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i_start)?;
        if let Some(attr) = attr {
            width = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("height:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            height = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("samples_per_pixel:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            samples_per_pixel = Some(attr.0);
        }
        let (i, attr) = opt(preceded(
            space_delimited(tag("max_depth:")),
            pair(space_delimited(expr), space_delimited(tag(","))),
        ))(i)?;
        if let Some(attr) = attr {
            max_depth = Some(attr.0);
        }
        let (i, res) = opt(space_delimited(close_brace))(i)?;
        i_start = i;
        if res.is_some() {
            break;
        }
    }

    if max_depth.is_none() || samples_per_pixel.is_none() || width.is_none() || height.is_none() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i_start,
            code: nom::error::ErrorKind::Tag,
        }));
    }
    Ok((
        i_start,
        (Statement::Config {
            span: calc_offset(i0, i_start),
            width: width.unwrap(),
            height: height.unwrap(),
            samples_per_pixel: samples_per_pixel.unwrap(),
            max_depth: max_depth.unwrap(),
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

pub fn parse<'a>(i: &'a str) -> Result<AST<'a>, Box<dyn Error + 'a>> {
    let i = Span::new(i);
    let res = statements_finish(i)?;
    Ok(res)
}
