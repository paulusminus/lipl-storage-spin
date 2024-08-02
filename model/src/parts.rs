use std::{
    io::{BufRead, BufReader, Error, Read},
    iter::once,
    ops::Add,
    str::FromStr,
};

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Debug, PartialEq)]
pub struct Parts(Vec<Vec<String>>);

impl Default for Parts {
    fn default() -> Self {
        vec![].into()
    }
}

impl Add<Vec<String>> for Parts {
    type Output = Parts;
    fn add(self, rhs: Vec<String>) -> Self::Output {
        self.0
            .into_iter()
            .chain(once(rhs))
            .collect::<Vec<_>>()
            .into()
    }
}

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

impl FromStr for Parts {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        fn next_part(lines: impl Iterator<Item = Result<String>>) -> Result<Vec<String>> {
            lines
                .map(|line| line.map(|l| l.trim().to_owned()))
                .skip_while(is_empty(true))
                .take_while(is_empty(false))
                .collect::<Result<Vec<String>>>()
        }

        fn parts_from_reader(
            mut lines: impl Iterator<Item = Result<String>>,
            parts: Parts,
        ) -> Result<Parts> {
            let part = next_part(&mut lines)?;
            if part.is_empty() {
                Ok(parts)
            } else {
                parts_from_reader(lines, parts + part)
            }
        }

        parts_from_reader(lines(s.as_bytes()), Parts::default())
    }
}

impl std::fmt::Display for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

#[cfg(test)]
mod test {
    use super::Parts;

    #[test]
    fn parse_part() {
        let test = "\r  Hallo allema  \t";
        assert_eq!(
            test.parse::<Parts>().unwrap(),
            Parts::from(vec![vec!["Hallo allema".to_owned()]]),
        );
    }

    #[test]
    fn parse_part2() {
        let test = "\rHallo allema\n \t\nJaJa\r\nNee  \t";
        assert_eq!(
            test.parse::<Parts>().unwrap(),
            Parts::from(vec![
                vec!["Hallo allema".to_owned()],
                vec!["JaJa".to_owned(), "Nee".to_owned()]
            ])
        );
    }

    #[test]
    fn sanitize() {
        let test = "\rHallo allema\n \t\nJaJa\r\nNee  \t";
        assert_eq!(
            test.parse::<Parts>().unwrap().to_text(),
            *"Hallo allema\n\nJaJa\nNee",
        );
    }
}
