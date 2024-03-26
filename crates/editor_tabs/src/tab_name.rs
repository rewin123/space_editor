/// This module contains the tab naming logic
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

pub trait TabName: Debug + Any {
    fn clear_background(&self) -> bool;
    fn title(&self) -> String;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct TabNameHolder {
    pub value: String,
    pub type_id: TypeId,
    pub clear_background: bool,
    pub title: String,
}

impl TabNameHolder {
    pub fn new<T: TabName>(value: T) -> Self {
        Self {
            value: format!("{:?}", value),
            type_id: TypeId::of::<T>(),
            clear_background: value.clear_background(),
            title: value.title(),
        }
    }
}

impl<T: TabName> From<T> for TabNameHolder {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestTabName(String);

    impl TabName for TestTabName {
        fn clear_background(&self) -> bool {
            false
        }

        fn title(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn test_tab_name_holder() {
        let holder = TabNameHolder::new(TestTabName("test".to_string()));

        assert_eq!(holder.value, "TestTabName(\"test\")");
        assert_eq!(holder.type_id, TypeId::of::<TestTabName>());
        assert_eq!(holder.clear_background, false);
        assert_eq!(holder.title, "test");
    }
}
