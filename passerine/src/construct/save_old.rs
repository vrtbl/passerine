impl TryFrom<AST> for ArgPattern {
    type Error = String;

    /// Like `ASTPattern`s, `ArgPattern`s are represented as ASTs,
    /// Then converted into `ArgPattern`s when the compiler determines it so.
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        Ok(
            match ast {
                AST::Symbol(s) => ArgPattern::Symbol(s),
                AST::ArgPattern(p) => p,
                AST::Form(f) => {
                    let mut mapped = vec![];
                    for a in f { mapped.push(a.map(ArgPattern::try_from)?); }
                    ArgPattern::Group(mapped)
                }
                _ => Err("Unexpected construct inside argument pattern")?,
            }
        )
    }
}

impl TryFrom<AST> for ASTPattern {
    type Error = String;

    /// Tries to convert an `AST` into a `CSTPattern`.
    /// CSTPatterns mirror the `AST`s they are designed to destructure.
    /// During parsing, they are just parsed as `AST`s -
    /// When the compiler can determine that an AST is actually a pattern,
    /// It performs this conversion.
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        Ok(
            match ast {
                AST::Symbol(s) => ASTPattern::Symbol(s),
                AST::Data(d) => ASTPattern::Data(d),
                AST::Label(k, a) => ASTPattern::Label(k, Box::new(a.map(ASTPattern::try_from)?)),
                AST::Pattern(p) => p,
                AST::Form(f) => {
                    let mut patterns = vec![];
                    for item in f {
                        patterns.push(item.map(ASTPattern::try_from)?);
                    }
                    ASTPattern::Chain(patterns)
                },
                AST::Tuple(t) => {
                    let mut patterns = vec![];
                    for item in t {
                        patterns.push(item.map(ASTPattern::try_from)?);
                    }
                    ASTPattern::Tuple(patterns)
                }
                AST::Group(e) => e.map(ASTPattern::try_from)?.item,
                _ => Err("Unexpected construct inside pattern")?,
            }
        )
    }
}

impl TryFrom<ASTPattern> for CSTPattern {
    type Error = String;

    /// Directly maps `ASTPattern`s to `CSTPattern`s.
    /// This function may become a bit more complex once 'where' is added.
    fn try_from(ast_pattern: ASTPattern) -> Result<Self, Self::Error> {
        Ok(
            match ast_pattern {
                ASTPattern::Symbol(s)   => CSTPattern::Symbol(s),
                ASTPattern::Data(d)     => CSTPattern::Data(d),
                ASTPattern::Label(k, a) => CSTPattern::Label(k, Box::new(a.map(CSTPattern::try_from)?)),
                ASTPattern::Tuple(t)    => CSTPattern::Tuple(t.into_iter().map(|i| i.map(CSTPattern::try_from)).collect::<Result<Vec<_>, _>>()?),
                ASTPattern::Chain(_)    => Err("Unexpected chained construct inside pattern")?,
            }
        )
    }
}
