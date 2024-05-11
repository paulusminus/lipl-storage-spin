use std::{
    io::{BufRead, BufReader, Error, Read},
    str::FromStr,
};

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct Parts(Vec<Vec<String>>);

impl From<Vec<Vec<String>>> for Parts {
    fn from(value: Vec<Vec<String>>) -> Self {
        Parts(value)
    }
}

impl Parts {
    pub fn parts(&self) -> Vec<Vec<String>> {
        self.0.clone()
    }

    pub fn to_text(&self) -> String {
        self.0
            .iter()
            .map(|part| part.join("\n"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

fn lines<R>(r: R) -> impl Iterator<Item = Result<String>>
where
    R: Read,
{
    BufReader::new(r).lines()
}

fn is_empty(b: bool) -> impl Fn(&Result<String>) -> bool {
    move |r| r.as_ref().ok().map(|s| s.is_empty()) == Some(b)
}

fn next_part(lines: impl Iterator<Item = Result<String>>) -> Result<Vec<String>> {
    lines
        .map(|line| line.map(|l| l.trim().to_owned()))
        .skip_while(is_empty(true))
        .take_while(is_empty(false))
        .collect::<Result<Vec<String>>>()
}

impl FromStr for Parts {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parts_from_reader(s.as_bytes())
    }
}

impl std::fmt::Display for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

fn parts_from_reader<R>(r: R) -> Result<Parts>
where
    R: Read,
{
    let mut lines = lines(r);
    let mut result = vec![];
    let mut part = next_part(&mut lines)?;
    while !part.is_empty() {
        result.push(part);
        part = next_part(&mut lines)?;
    }

    Ok(result.into())
}

#[cfg(test)]
mod test {
    use super::parts_from_reader;

    #[test]
    fn from_reader() {
        let test = "\r  Hallo allema  \t";
        assert_eq!(
            parts_from_reader(test.as_bytes()).unwrap().parts(),
            vec![vec!["Hallo allema".to_owned()]]
        );
    }

    #[test]
    fn from_reader2() {
        let test = "\rHallo allema\n \t\nJaJa\r\nNee  \t";
        assert_eq!(
            parts_from_reader(test.as_bytes()).unwrap().parts(),
            vec![
                vec!["Hallo allema".to_owned()],
                vec!["JaJa".to_owned(), "Nee".to_owned()]
            ]
        );
    }

    #[test]
    fn sanitize() {
        let test = "\rHallo allema\n \t\nJaJa\r\nNee  \t";
        assert_eq!(
            parts_from_reader(test.as_bytes()).unwrap().to_text(),
            *"Hallo allema\n\nJaJa\nNee",
        );
    }
}
