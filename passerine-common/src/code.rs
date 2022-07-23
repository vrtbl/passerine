#[derive(Default)]
pub struct Store {
    data: Vec<Data>,
    code: Vec<u8>,
    funs: Vec<FunInfo>,
    spans: Vec<(usize, Span)>,
}

struct FunInfo {
    offset: usize,
    length: usize,
    locals: usize,
    caps: Vec<Capture>,
}

struct FunIndex(usize);

impl Store {
    pub fn empty() -> Store {
        Default::default()
    }

    pub fn add_data(&mut self, data: Data) -> usize {
        let index = self.data.len();
        self.data.append(data);
        return index;
    }

    pub fn add_function(
        &mut self,
        locals: usize,
        caps: Vec<Capture>,
        bytecode: Vec<u8>,
        spans: Vec<Span>,
    ) -> usize {
        let offset = self.code.len();
        let length = bytecode.len();
        self.code.push(bytecode);

        let index = self.funs.len();
        self.funs.push(FunInfo {
            offset,
            length,
            locals,
            caps,
        });
        return index;
    }
}
