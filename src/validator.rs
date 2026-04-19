use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::error::Error;

pub fn validate<T: Validate>(schema: &T) -> Result<(), Error> {
    match schema.validate() {
        Ok(_) => Ok(()),
        Err(err) => Err(map_validation_err(err)),
    }
}

fn map_validation_err(errors: ValidationErrors) -> Error {
    if let Some((path, error)) = first_validation_error(&errors, String::new()) {
        return Error::BadRequest(format_validation_message(&path, error));
    }

    Error::BadRequest("Validation failed".to_string())
}

fn first_validation_error<'a>(
    errors: &'a ValidationErrors,
    parent_path: String,
) -> Option<(String, &'a ValidationError)> {
    let mut fields: Vec<_> = errors.errors().iter().collect();
    fields.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (field, kind) in fields {
        let path = if parent_path.is_empty() {
            field.to_string()
        } else {
            format!("{}.{}", parent_path, field)
        };

        match kind {
            ValidationErrorsKind::Field(field_errors) => {
                if let Some(error) = field_errors.first() {
                    return Some((path, error));
                }
            }
            ValidationErrorsKind::Struct(nested) => {
                if let Some(found) = first_validation_error(nested, path) {
                    return Some(found);
                }
            }
            ValidationErrorsKind::List(items) => {
                let mut indices: Vec<_> = items.iter().collect();
                indices.sort_by_key(|(index, _)| *index);

                for (index, nested) in indices {
                    let list_path = format!("{}[{}]", path, index);
                    if let Some(found) = first_validation_error(nested, list_path) {
                        return Some(found);
                    }
                }
            }
        }
    }

    None
}

fn format_validation_message(field_path: &str, error: &ValidationError) -> String {
    if let Some(message) = &error.message {
        return format!("{} {}", field_path, message);
    }

    match error.code.as_ref() {
        "length" => {
            let min = error.params.get("min");
            let max = error.params.get("max");
            let equal = error.params.get("equal");

            if let Some(equal) = equal {
                return format!("{} length must be {}", field_path, equal);
            }
            if let (Some(min), Some(max)) = (min, max) {
                return format!("{} length must be between {} and {}", field_path, min, max);
            }
            if let Some(min) = min {
                return format!("{} length must be at least {}", field_path, min);
            }
            if let Some(max) = max {
                return format!("{} length must be at most {}", field_path, max);
            }
            format!("{} has invalid length", field_path)
        }
        "email" => format!("{} must be a valid email address", field_path),
        "required" => format!("{} is required", field_path),
        code => format!("{} is invalid ({})", field_path, code),
    }
}
