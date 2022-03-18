pub enum EffectCore {
    Fatal,
    WriteStdOut(String),
    WriteStdErr(String),
    ReadStdIn(String),
    ToString(Data),
}
