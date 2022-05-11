use crate::{
    common::{
        Span,
        Spanned,
    },
    construct::{
        symbol::SharedSymbol,
        tree::{
            Base,
            Lambda,
            Sugar,
            AST,
            CST,
        },
    },
};

pub struct Desugarer;

type SharedBase<T> = Base<Spanned<T>, SharedSymbol>;

impl Desugarer {
    pub fn desugar(ast: Spanned<AST>) -> Spanned<CST> {
        todo!();
    }

    fn walk(ast: Spanned<AST>) -> Spanned<CST> {
        // TODO: use this destructuring pattern throughout codebase!
        let Spanned { item, span } = ast;
        let item = match item {
            AST::Base(b) => CST::Base(Desugarer::walk_base(b)),
            AST::Lambda(l) => CST::Lambda(Desugarer::walk_lambda(l)),
            AST::Sugar(s) => Desugarer::walk_sugar(s),
        };
        return Spanned::new(item, span);
    }

    fn walk_base(b: SharedBase<AST>) -> SharedBase<CST> {
        match b {
            Base::Symbol(_) => todo!(),
            Base::Label(_) => todo!(),
            Base::Lit(_) => todo!(),
            Base::Tuple(_) => todo!(),
            Base::Module(_) => todo!(),
            Base::Block(_) => todo!(),
            Base::Call(_, _) => todo!(),
            Base::Assign(_, _) => todo!(),
            Base::FFI(_, _) => unreachable!("FFI is depracated... :("),
        }
    }

    fn walk_lambda(l: Lambda<Spanned<AST>>) -> Lambda<Spanned<CST>> {
        let Lambda { arg, body } = l;
        let body = Desugarer::walk(*body);
        return Lambda::new(arg, body);
    }

    fn walk_sugar(s: Sugar<Spanned<AST>>) -> CST {
        match s {
            Sugar::Group(g) => Desugarer::walk(*g).item,
            // TODO: just do this during parsing haha
            // turn a form into a call:
            Sugar::Form(f) => {
                // we know the form can not be empty...
                // and must have at least two items...
                assert!(f.len() >= 2);
                let mut form_items = f.into_iter().rev();
                let mut fun = Desugarer::walk(form_items.next().unwrap());

                for arg in form_items {
                    let arg = Desugarer::walk(arg);
                    let span = Span::combine(&fun.span, &arg.span);
                    let call = SharedBase::call(fun, arg);
                    fun = Spanned::new(CST::Base(call), span);
                }

                fun.item
            },
            // TODO: don't ignore type annotations!
            Sugar::Is(e, _) => {
                unimplemented!("type annotations will be implemented when the type checker is implemented")
            },
            Sugar::Comp(arg, fun) => CST::Base(Base::call(
                Desugarer::walk(*fun),
                Desugarer::walk(*arg),
            )),
            Sugar::Field(_, _) => unimplemented!(
                "field access will be implemented when structs are implemented"
            ),
        }
    }
}
