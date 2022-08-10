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

macro_rules! bool_data {
    ($data:ident) => {
        if let Data::Bool(b) = $data {
            b
        } else {
            return Err(RuntimeError::TypeMismatched(
                $data.clone(),
                "bool".to_string(),
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

macro_rules! check_memory_bound {
    ($memory:expr, $offset:expr, $type:expr) => {
        let bytes = match $type {
            Type::Float => 4,
            Type::Bool => 1,
        };
        let actual_bytes = $memory.len() - $offset;

        if !bytes <= actual_bytes {
            return Err(RuntimeError::OutOfMemory($offset, $type));
        }
    };
}

pub(crate) use bool_data;
pub(crate) use check_memory_bound;
pub(crate) use float_data;
pub(crate) use stack_pop;
