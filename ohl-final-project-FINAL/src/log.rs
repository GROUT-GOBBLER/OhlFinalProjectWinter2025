pub struct Log {
    indent: usize,
    pub show_debug: bool,
}

impl Log {

    const INDENT : usize = 2;

    pub fn new() -> Log {
        Log { indent: 0, show_debug: true, }
    }

    pub fn debug(&self, msg: &str) {
        if self.show_debug {
            println!("{:<indent$}{:}", "", msg, indent=self.indent);
        }
    }

    pub fn indent_inc(&mut self) {
        self.indent += Self::INDENT;
    }
    pub fn indent_dec(&mut self) {
        self.indent -= Self::INDENT;
    }
}