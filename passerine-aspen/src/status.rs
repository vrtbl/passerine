use colored::*;

pub enum Kind {
    Info,
    Success,
    Warn,
    Fatal,
}

pub struct Status(pub Kind, pub &'static str);

impl Status {
    pub fn info() -> Status {
        Status(Kind::Info, "Info")
    }
    pub fn success() -> Status {
        Status(Kind::Success, "Success")
    }
    pub fn warn() -> Status {
        Status(Kind::Warn, "Warning")
    }
    pub fn fatal() -> Status {
        Status(Kind::Fatal, "Fatal")
    }

    fn tag(&self) -> ColoredString {
        match self.0 {
            Kind::Info => self.1.blue(),
            Kind::Success => self.1.green(),
            Kind::Warn => self.1.yellow(),
            Kind::Fatal => self.1.red(),
        }
        .bold()
    }

    fn multiline(&self, lines: Vec<&str>) {
        eprint!("\n{} ", self.tag());
        // let blank   = " ".repeat(tag.len()).hidden();
        for line in lines {
            eprintln!("{}", line);
        }
        eprintln!()
    }

    pub fn log(&self, message: &str) {
        let lines = message.lines().collect::<Vec<&str>>();

        if lines.len() > 1 {
            self.multiline(lines);
        } else {
            eprintln!("{:>12} {}", self.tag(), message);
        }
    }
}
