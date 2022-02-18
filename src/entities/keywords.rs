#[derive(Debug)]
pub struct Keywords {
    items: Vec<String>,
}

impl Keywords {
    pub fn as_string(&self) -> String {
        self.items.join(" ")
    }
}

impl<T, I> From<T> for Keywords
where
    T: IntoIterator<Item = I>,
    I: Into<String>,
{
    fn from(items: T) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

impl AsRef<[String]> for Keywords {
    fn as_ref(&self) -> &[String] {
        self.items.as_ref()
    }
}
