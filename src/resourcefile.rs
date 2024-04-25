use std::path::{Path, PathBuf};

pub struct Resourceproto<'a, T: 'a + askama::Template> {
    pub name: &'a str,
    pub prototype: T,
}

pub struct Resourcefile {
    pub name: String,
    pub buffer: String,
}

impl<'a, T: 'a + askama::Template> Resourceproto<'a, T> {
    pub fn render(self) -> Resourcefile {
        Resourcefile {
            name: self.name.to_owned(),
            buffer: self.prototype.render().unwrap(),
        }
    }
}

impl Resourcefile {
    fn ensure_base(path: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(path)
    }

    pub fn write(&self, mut path: PathBuf) -> Result<(), std::io::Error> {
        path.push(&self.name);
        Resourcefile::ensure_base(path.parent().unwrap())?;
        println!("Writing to {}", &path.to_string_lossy());
        std::fs::write(path, &self.buffer)?;
        Ok(())
    }

    pub fn append(&mut self, other: Resourcefile) {
        self.buffer
            .push_str(&format!("\n\n# {}\n---\n", other.name));
        self.buffer.push_str(&other.buffer);
    }
}
