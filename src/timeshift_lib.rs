use chrono::NaiveDateTime;
use std::fmt;
use std::process::Command;
use std::str;
#[derive(Debug, Default)]
pub struct Snapshot {
    num: u8,

    date: NaiveDateTime, // Pour l'instant on utilise une string pour la date

    tags: char,
    description: String,
}

impl Snapshot {
    pub fn new(num: u8, name: &str, tags: char, description: String) -> Self {
        let date = NaiveDateTime::parse_from_str(name, "%Y-%m-%d_%H-%M-%S")
            .expect("Could not parse a date from the name of the snapshot");
        Snapshot {
            num,
            date,
            tags,
            description,
        }
    }
}
impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {}",
            self.num,
            self.date.format("%Y-%m-%d %H:%M:%S"),
            self.tags,
            self.description
        )
    }
}

#[derive(Debug, Default)]
pub struct Timeshift {
    pub snapshots: Vec<Snapshot>,
}

impl Timeshift {
    pub fn new() -> Self {
        let snapshots = Self::get_snapshots();
        Timeshift { snapshots }
    }
    pub fn get_snapshots() -> Vec<Snapshot> {
        let mut result: Vec<Snapshot> = Vec::new();
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list")
            .output()
            .expect("Couldn't get snapshots list");
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(10) {
            //getto, à améliorer
            if line.is_empty() {
                break;
            }
            result.push(Self::parse_output(line));
        }
        result
    }
    pub fn parse_output(output: &str) -> Snapshot {
        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() < 5 {
            // panic!("Invalid snapshot format");
        }
        let num = parts[0].parse::<u8>().expect("Could not parse num");
        let name = parts[2];
        let tags = parts[3].parse::<char>().expect("could not parse Tags");
        let description = parts[4..].join(" ").to_string();
        // On récupère tout le
        // reste (heuresement que
        // le truc est à la fin)

        Snapshot::new(num, name, tags, description)
    }
}
