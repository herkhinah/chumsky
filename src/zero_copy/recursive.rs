use super::*;

enum RecursiveInner<T: ?Sized> {
    Owned(Rc<T>),
    Unowned(Weak<T>),
}

type OnceParser<'a, I, O, E, S> = OnceCell<Box<dyn Parser<'a, I, O, E, S> + 'a>>;

pub type Direct<'a, I, O, E, S = ()> = dyn Parser<'a, I, O, E, S> + 'a;

pub struct Indirect<'a, I: ?Sized, O, E, S = ()> {
    inner: OnceCell<Box<dyn Parser<'a, I, O, E, S> + 'a>>,
}

pub struct Recursive<P: ?Sized> {
    inner: RecursiveInner<P>,
}

impl<'a, I: Input + ?Sized, O, E: Error<I>, S> Recursive<Indirect<'a, I, O, E, S>> {
    pub fn declare() -> Self {
        Recursive {
            inner: RecursiveInner::Owned(Rc::new(Indirect {
                inner: OnceCell::new(),
            })),
        }
    }

    pub fn define<P: Parser<'a, I, O, E, S> + 'a>(&mut self, parser: P) {
        self.parser()
            .inner
            .set(Box::new(parser))
            .unwrap_or_else(|_| panic!("recursive parser already declared"));
    }
}

impl<P: ?Sized> Recursive<P> {
    fn parser(&self) -> Rc<P> {
        match &self.inner {
            RecursiveInner::Owned(x) => x.clone(),
            RecursiveInner::Unowned(x) => x
                .upgrade()
                .expect("Recursive parser used before being defined"),
        }
    }
}

impl<P: ?Sized> Clone for Recursive<P> {
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                RecursiveInner::Owned(x) => RecursiveInner::Owned(x.clone()),
                RecursiveInner::Unowned(x) => RecursiveInner::Unowned(x.clone()),
            },
        }
    }
}

impl<'a, I, O, E, S> Parser<'a, I, O, E, S> for Recursive<Indirect<'a, I, O, E, S>>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, O, E> {
        M::invoke(
            self.parser()
                .inner
                .get()
                .expect("Recursive parser used before being defined")
                .as_ref(),
            inp,
        )
    }

    go_extra!(O);
}

impl<'a, I, O, E, S> Parser<'a, I, O, E, S> for Recursive<Direct<'a, I, O, E, S>>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, O, E> {
        M::invoke(&*self.parser(), inp)
    }

    go_extra!(O);
}

pub fn recursive<'a, I, O, E, S, A, F>(f: F) -> Recursive<Direct<'a, I, O, E, S>>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    A: Parser<'a, I, O, E, S> + 'a,
    F: FnOnce(Recursive<Direct<'a, I, O, E, S>>) -> A,
{
    let rc = Rc::new_cyclic(|rc| {
        let rc: Weak<dyn Parser<'a, I, O, E, S>> = rc.clone() as _;
        let parser = Recursive {
            inner: RecursiveInner::Unowned(rc.clone()),
        };

        f(parser)
    });

    Recursive {
        inner: RecursiveInner::Owned(rc),
    }
}
