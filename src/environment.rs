use expression::Expression;

pub trait Environment {
    fn get(&self, key: impl AsRef<str>) -> Option<&Expression>;
}
