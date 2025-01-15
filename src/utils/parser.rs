use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use ply_rs::parser::Parser;
use ply_rs::ply::{DefaultElement, Property};
use thiserror::Error;
use crate::hittable::hittable_list::HittableList;
use crate::value::material::Material;
use crate::utils::parser::ParseError::{Parse, ParseElement, ParseValue};
use crate::utils::parser::ParseValueError::ParseProperty;
use crate::hittable::triangle::Triangle;
use crate::value::vec3::Vec3;

pub fn parse_ply(path: &PathBuf, mat: Rc<dyn Material>) -> Result<HittableList, ParseError> {
    let mut file = File::open(path)?;
    // Create PLY parser
    let parser = Parser::<DefaultElement>::new();
    let ply = parser.read_ply(&mut file)?;

    let vertices = ply.payload.get("vertex")
        .ok_or(Parse("No vertices in PLY file".to_string()))?
        .iter().map(parse_vertex)
        .collect::<Result<Vec<_>, _>>()?;

    let faces = ply.payload.get("face")
        .ok_or(Parse("No faces in PLY file".to_string()))?
        .iter().map(|f| parse_face(f, &vertices, Rc::clone(&mat)))
        .collect::<Result<Vec<_>, _>>()?;

    let mut world = HittableList::default();
    for face in faces {
        world.add(Rc::new(face))
    }

    Ok(world)
}

fn parse_vertex(element: &DefaultElement) -> Result<Vec3, ParseError> {
    let x = parse_float(element.get("x")
        .ok_or(ParseElement(element.clone(), "Vertex has no x value".to_string()))?)
        .map_err(|e| ParseValue(element.clone(), e))?;
    let y = parse_float(element.get("y")
        .ok_or(ParseElement(element.clone(), "Vertex has no y value".to_string()))?)
        .map_err(|e| ParseValue(element.clone(), e))?;
    let z = parse_float(element.get("z")
        .ok_or(ParseElement(element.clone(), "Vertex has no z value".to_string()))?)
        .map_err(|e| ParseValue(element.clone(), e))?;
    Ok(Vec3::new(x, y, z))
    // Ok(Vec3::new(x, z, y))
    // Ok(Vec3::new(y, x, z))
    // Ok(Vec3::new(y, z, x))
    // Ok(Vec3::new(z, x, y))
    // Ok(Vec3::new(z, y, x))
}

fn parse_face(element: &DefaultElement, vertices: &[Vec3], mat: Rc<dyn Material>) -> Result<Triangle, ParseError> {
    let vertices = parse_list(element.get("vertex_indices")
        .ok_or(ParseElement(element.clone(), "Face has no vertex_indices value".to_string()))?)
        .map_err(|e| ParseValue(element.clone(), e))?
        .iter().map(|idx| { vertices.get(*idx as usize).ok_or(ParseElement(element.clone(), format!("Vertex {idx} not found"))) })
        .collect::<Result<Vec<_>, _>>()?;
    if vertices.len() != 3 { return Err(ParseElement(element.clone(), "Face should have 3 vertices".to_string())); }
    Ok(Triangle::new(*vertices[0], *vertices[1], *vertices[2], mat))
}

fn parse_float(property: &Property) -> Result<f64, ParseValueError> {
    if let Property::Float(float) = property {
        Ok(*float as f64 * 100.0)
    } else {
        Err(ParseProperty("Property is not a double value.".to_string()))
    }
}

fn parse_list(property: &Property) -> Result<&Vec<i32>, ParseValueError> {
    if let Property::ListInt(list) = property {
        Ok(list)
    } else {
        Err(ParseProperty("Property is not a double value.".to_string()))
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Error opening file")]
    IO(#[from] io::Error),
    #[error("Parse error {0}")]
    Parse(String),
    #[error("Parse error in {0:?}: {1}")]
    ParseElement(DefaultElement, String),
    #[error("Parse error in {0:?}: {1}")]
    ParseValue(DefaultElement, #[source] ParseValueError),
}

#[derive(Error, Debug)]
pub enum ParseValueError {
    #[error("Parse error {0}")]
    ParseProperty(String),
}
