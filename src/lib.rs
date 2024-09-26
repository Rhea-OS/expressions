pub mod error;


pub struct Address {
    
}

pub struct Value {
    
}

pub trait DataSource {
    fn get_value(value: Address) -> Option<Value>;
}

#[cfg(test)]
mod test {
    use crate::error::*;

    #[test]
    pub fn test() -> Result<()> {
        Ok(())
    }
}