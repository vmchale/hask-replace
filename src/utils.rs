pub fn from_vec<T>(x: Option<Vec<T>>) -> Vec<T> {
    x.unwrap_or(vec![])
}

pub trait AsString {
    fn as_string(self) -> String;
}

impl<'a> AsString for Vec<String> {
    fn as_string(self) -> String {
        self.iter().fold("".to_string(), |acc, x| acc + x)
    }
}

impl<'a> AsString for &'a str {
    fn as_string(self) -> String {
        self.to_string()
    }
}

impl<'a> AsString for Vec<&'a str> {
    fn as_string(self) -> String {
        concat_str(self)
    }
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl<'a> IsEmpty for &'a str {
    fn is_empty(&self) -> bool {
        self == &""
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn join<T: IsEmpty>(xs: Vec<Vec<T>>) -> Vec<T> {
    let mut v = Vec::new();
    for x in xs {
        for inner in x {
            if !inner.is_empty() {
                v.push(inner);
            }
        }
    }
    v
}

#[allow(dead_code)]
pub fn concat_str(xs: Vec<&str>) -> String {
    xs.into_iter().fold("".to_string(), |acc, x| acc + x)
}

pub fn swap_module<'a>(old: &'a str, new: &'a str, candidate: &'a str) -> &'a str {
    if candidate == old { new } else { candidate }
}
