use super::Line;

pub fn lines<'a>(string: &'a str) -> impl Iterator<Item = Line<'a>> + 'a {
    string.trim_matches('\n').split('\n').map(Line::from)
}
