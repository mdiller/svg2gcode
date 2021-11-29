#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use euclid::default::Box2D;
use g_code::emit::{
    command::{ABSOLUTE_DISTANCE_MODE_FIELD, RELATIVE_DISTANCE_MODE_FIELD},
    Field, Token, Value,
};
use lyon_geom::{point, vector, Point};

type F64Point = Point<f64>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PostprocessConfig {
    #[deprecated(
        since = "0.7.0",
        note = "Setting the origin is now a preprocessing operation"
    )]
    pub origin: [f64; 2],
}

/// Moves all the commands so that they are beyond a specified position
#[deprecated(
    since = "0.7.0",
    note = "Setting the origin is now a preprocessing operation"
)]
pub fn set_origin(tokens: &mut [Token<'_>], origin: [f64; 2]) {
    let offset =
        -get_bounding_box(tokens.iter()).min.to_vector() + F64Point::from(origin).to_vector();

    let mut is_relative = false;
    let mut current_position = point(0f64, 0f64);
    let mut should_skip = false;
    for token in tokens {
        match token {
            abs if *abs == Token::Field(ABSOLUTE_DISTANCE_MODE_FIELD) => is_relative = false,
            rel if *rel == Token::Field(RELATIVE_DISTANCE_MODE_FIELD) => is_relative = true,
            // Don't edit M codes for relativity
            Token::Field(Field { letters, .. }) if *letters == "M" => should_skip = true,
            Token::Field(Field { letters, .. }) if *letters == "G" => should_skip = false,
            Token::Field(Field { letters, value }) if *letters == "X" && !should_skip => {
                if let Some(float) = value.as_f64() {
                    if is_relative {
                        current_position += vector(float, 0.)
                    } else {
                        current_position = point(float, 0.);
                    }
                    *value = Value::Float(current_position.x + offset.x)
                }
            }
            Token::Field(Field { letters, value }) if *letters == "Y" && !should_skip => {
                if let Some(float) = value.as_f64() {
                    if is_relative {
                        current_position += vector(0., float)
                    } else {
                        current_position = point(0., float);
                    }
                    *value = Value::Float(current_position.y + offset.y)
                }
            }
            _ => {}
        }
    }
}

fn get_bounding_box<'a, I: Iterator<Item = &'a Token<'a>>>(tokens: I) -> Box2D<f64> {
    let (mut minimum, mut maximum) = (point(0f64, 0f64), point(0f64, 0f64));
    let mut is_relative = false;
    let mut should_skip = false;
    let mut current_position = point(0f64, 0f64);
    for token in tokens {
        match token {
            abs if *abs == Token::Field(ABSOLUTE_DISTANCE_MODE_FIELD) => is_relative = false,
            rel if *rel == Token::Field(RELATIVE_DISTANCE_MODE_FIELD) => is_relative = true,
            // Don't check M codes for relativity
            Token::Field(Field { letters, .. }) if *letters == "M" => should_skip = true,
            Token::Field(Field { letters, .. }) if *letters == "G" => should_skip = false,
            Token::Field(Field { letters, value }) if *letters == "X" && !should_skip => {
                if let Some(value) = value.as_f64() {
                    if is_relative {
                        current_position += vector(value, 0.)
                    } else {
                        current_position = point(value, 0.);
                    }
                    minimum = minimum.min(current_position);
                    maximum = maximum.max(current_position);
                }
            }
            Token::Field(Field { letters, value }) if *letters == "Y" && !should_skip => {
                if let Some(value) = value.as_f64() {
                    if is_relative {
                        current_position += vector(0., value)
                    } else {
                        current_position = point(0., value);
                    }
                    minimum = minimum.min(current_position);
                    maximum = maximum.max(current_position);
                }
            }
            _ => {}
        }
    }
    Box2D::new(minimum, maximum)
}
