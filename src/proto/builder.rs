#![allow(dead_code, unused_variables)]

use std::ops::{Deref, DerefMut};

pub struct ReusableBuilder {
    val: usize,
}

pub trait ReusableBuilderTrait
where
    Self: Sized + Deref<Target = ReusableBuilder> + DerefMut<Target = ReusableBuilder>,
{
    fn increment(mut self) -> Self {
        (*self).val += 1;
        self
    }
}
pub struct MyBuilder {
    val: String,
    builder: ReusableBuilder,
}

impl Deref for MyBuilder {
    type Target = ReusableBuilder;
    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for MyBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

impl ReusableBuilderTrait for MyBuilder {}

impl MyBuilder {
    pub fn new(val: &str) -> Self {
        Self {
            val: val.to_string(),
            builder: ReusableBuilder { val: 0 },
        }
    }

    pub fn out(self) -> String {
        format!("val: {} - {}", self.val, self.builder.val)
    }
}
