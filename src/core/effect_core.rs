pub enum Core {
    Fatal,
    WriteOut(String),
    WriteErr(String),
    ReadIn(String),
    ToString(Data),
}

pub trait Effect {
    fn from_data(data: Data) -> Result<Self, String>;
}

pub struct EffectSet<T> {
    effects: HashMap,
}

pub fn main() {
    effect_set! {
        enum E {
            "std.Core" => Core,
            "std.http.IO" => Http,
        }
    };
}
