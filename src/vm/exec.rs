pub struct Func {
    code:  Vec<u8>,
    spans: Vec<(usize, Span)>,
    args:   usize,
    locals: usize,
}

pub struct Exec {
    funcs: Vec<Func>,
    data:  Vec<Data>,
}

pub struct Fiber {
    stack: Stack,
    heap:  Vec<u64>, // TODO: heap type
    func:  usize,
    index: usize,
}

impl Fiber {
    pub fn new(func: usize) -> Fiber {
        Fiber {
            stack: Stack::new(),
            heap:  vec![],
            func,
            index: 0,
        }
    }
}

pub struct VM {
    fibers: Vec<Fiber>,
    active: Fiber,
    exec:   Exec,
}

impl VM {
    pub fn new(exec: Exec) -> VM {
        VM {
            fibers: vec![],
            active: Fiber::new(0),
            exec,
        }
    }

    pub fn next_effect() -> (usize, Data) {
        todo!()
    }
}