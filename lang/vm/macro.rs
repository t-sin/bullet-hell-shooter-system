macro_rules! float_data {
    ($data:ident) => {
        if let Data::Float(f) = $data {
            f
        } else {
            return Err(RuntimeError::TypeMismatched(
                $data.clone(),
                "float".to_string(),
            ));
        }
    };
}

pub(crate) use float_data;
