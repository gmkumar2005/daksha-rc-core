pub mod definitions_manager;

pub fn add_definitions_manager(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add_definitions_manager(2, 2);
        assert_eq!(result, 4);
    }
}
