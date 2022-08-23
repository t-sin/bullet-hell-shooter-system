use crate::Data;
use lang_component::syntax::Type;

pub trait State {
    fn get(&self, id: usize) -> Option<Data>;
    fn set(&mut self, id: usize, d: Data) -> Result<(), Option<Type>>;
}
