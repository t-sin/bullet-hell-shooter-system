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

macro_rules! stack_pop {
    ($stack:expr) => {
        if let Some(d) = $stack.pop() {
            d
        } else {
            return Err(RuntimeError::StackUnderflow);
        }
    };
}

pub(crate) use float_data;
pub(crate) use stack_pop;