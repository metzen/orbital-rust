use super::*;

#[derive(Debug, PartialEq)]
struct Thing {
    value: u8,
}

impl Thing {
    fn new(value: u8) -> Thing {
        Thing { value }
    }
}

#[test]
fn test_minmax_by_key() {
    let result = minmax_by_key(Thing::new(3), Thing::new(1), |x| x.value);
    assert_eq!(result, [Thing::new(1), Thing::new(3)]);
}
