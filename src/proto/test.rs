use super::builder::{MyBuilder, ReusableBuilderTrait};

#[test]
fn test_builder() {
    let builder = MyBuilder::new("hi");
    let val = builder.increment().out();
    assert_eq!(val, "val: hi - 1");
}
