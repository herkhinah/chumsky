use super::*;

pub struct End<I: ?Sized>(PhantomData<I>, #[cfg(debug_assertions)] Location<'static>);

#[track_caller]
pub const fn end<I: Input + ?Sized>() -> End<I> {
    End(PhantomData, #[cfg(debug_assertions)] *Location::caller())
}

impl<I: ?Sized> Copy for End<I> {}
impl<I: ?Sized> Clone for End<I> {
    fn clone(&self) -> Self {
        End(PhantomData, #[cfg(debug_assertions)] self.1)
    }
}

impl<'a, I, E, S> Parser<'a, I, E, S> for End<I>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    type Output = ();

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let before = inp.save();
        match inp.next() {
            (_, None) => Ok(M::bind(|| ())),
            (at, Some(tok)) => Err(Located::at(
                at,
                E::expected_found(None, Some(tok), inp.span_since(before)),
            )),
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("end", self.1) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { Some(0).. Some(0) }

    go_extra!();
}

pub struct Empty<I: ?Sized>(PhantomData<I>, #[cfg(debug_assertions)] Location<'static>);

#[track_caller]
pub const fn empty<I: Input + ?Sized>() -> Empty<I> {
    Empty(PhantomData, #[cfg(debug_assertions)] *Location::caller())
}

impl<I: ?Sized> Copy for Empty<I> {}
impl<I: ?Sized> Clone for Empty<I> {
    fn clone(&self) -> Self {
        Empty(PhantomData, #[cfg(debug_assertions)] self.1)
    }
}

impl<'a, I, E, S> Parser<'a, I, E, S> for Empty<I>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    type Output = ();

    fn go<M: Mode>(&self, _: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        Ok(M::bind(|| ()))
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("empty", self.1) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { Some(0).. Some(0) }

    go_extra!();
}

pub trait Seq<T> {
    type Iter<'a>: Iterator<Item = T>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

impl<T: Clone> Seq<T> for T {
    type Iter<'a> = core::iter::Once<T>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::iter::once(self.clone())
    }
}

impl<T: Clone, const N: usize> Seq<T> for [T; N] {
    type Iter<'a> = core::array::IntoIter<T, N>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new(self.clone())
    }
}

impl<'b, T: Clone, const N: usize> Seq<T> for &'b [T; N] {
    type Iter<'a> = core::array::IntoIter<T, N>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new((*self).clone())
    }
}

impl Seq<char> for str {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl<'b> Seq<char> for &'b str {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl Seq<char> for String {
    type Iter<'a> = core::str::Chars<'a>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

// impl<'b, T, C: Container<T>> Container<T> for &'b C {
//     type Iter<'a> = C::Iter<'a>;
//     fn iter(&self) -> Self::Iter<'_> { (*self).iter() }
// }

pub struct Just<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for Just<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for Just<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

#[track_caller]
pub const fn just<T, I, E, S>(seq: T) -> Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    Just {
        seq,
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = T;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let mut items = self.seq.iter();
        loop {
            match items.next() {
                Some(next) => {
                    let before = inp.save();
                    match inp.next() {
                        (_, Some(tok)) if next == tok => {}
                        (at, tok) => {
                            break Err(Located::at(
                                at,
                                E::expected_found(Some(Some(next)), tok, inp.span_since(before)),
                            ))
                        }
                    }
                }
                None => break Ok(M::bind(|| self.seq.clone())),
            }
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("just", self.location) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> {
        let seq_len = self.seq.iter().count();
        Some(seq_len)..Some(seq_len)
    }

    go_extra!();
}

pub struct OneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for OneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for OneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

#[track_caller]
pub const fn one_of<T, I, E, S>(seq: T) -> OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    OneOf {
        seq,
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().any(|not| not == tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(self.seq.iter().map(Some), found, inp.span_since(before)),
            )),
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("one_of", self.location) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { Some(1).. Some(1) }

    go_extra!();
}

pub struct NoneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for NoneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for NoneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

#[track_caller]
pub const fn none_of<T, I, E, S>(seq: T) -> NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    NoneOf {
        seq,
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().all(|not| not != tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(None, found, inp.span_since(before)),
            )),
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("none_of", self.location) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { Some(1).. Some(1) }

    go_extra!();
}

pub struct Any<I: ?Sized, E, S = ()> {
    phantom: PhantomData<(E, S, I)>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

impl<I: ?Sized, E, S> Copy for Any<I, E, S> {}
impl<I: ?Sized, E, S> Clone for Any<I, E, S> {
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

impl<'a, I, E, S> Parser<'a, I, E, S> for Any<I, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let before = inp.save();
        match inp.next() {
            (_, Some(tok)) => Ok(M::bind(|| tok)),
            (at, found) => Err(Located::at(
                at,
                E::expected_found(None, found, inp.span_since(before)),
            )),
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("any", self.location) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { Some(1).. Some(1) }

    go_extra!();
}

#[track_caller]
pub const fn any<I: Input + ?Sized, E: Error<I>, S>() -> Any<I, E, S> {
    Any {
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

pub struct TakeUntil<P, I: ?Sized, C = (), E = (), S = ()> {
    until: P,
    phantom: PhantomData<(C, E, S, I)>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

impl<'a, I, E, S, P, C> TakeUntil<P, I, C, E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, E, S>,
{
    pub fn collect<D: Container<P::Output>>(self) -> TakeUntil<P, D> {
        TakeUntil {
            until: self.until,
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

impl<P: Copy, I: ?Sized, C, E, S> Copy for TakeUntil<P, I, C, E, S> {}
impl<P: Clone, I: ?Sized, C, E, S> Clone for TakeUntil<P, I, C, E, S> {
    fn clone(&self) -> Self {
        TakeUntil {
            until: self.until.clone(),
            phantom: PhantomData,
            #[cfg(debug_assertions)] location: self.location,
        }
    }
}

#[track_caller]
pub const fn take_until<'a, P, I, E, S>(until: P) -> TakeUntil<P, I, (), E, S>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, E, S>,
{
    TakeUntil {
        until,
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

impl<'a, P, I, E, S, C> Parser<'a, I, E, S> for TakeUntil<P, C>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
    P: Parser<'a, I, E, S>,
    C: Container<I::Token>,
{
    type Output = (C, P::Output);

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let mut output = M::bind(|| C::default());

        loop {
            let start = inp.save();
            let e = match self.until.go::<M>(inp) {
                Ok(out) => break Ok(M::combine(output, out, |output, out| (output, out))),
                Err(e) => e,
            };

            inp.rewind(start);

            match inp.next() {
                (_, Some(tok)) => {
                    output = M::map(output, |mut output: C| {
                        output.push(tok);
                        output
                    })
                }
                (_, None) => break Err(e),
            }
        }
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("take_until", self.location) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> {
        let until_start = self.until.fp().start;
        if until_start == Some(0) {
            eprintln!("[parser problem]\n");
        }
        until_start..None
    }

    go_extra!();
}

pub struct Todo<I: ?Sized, E>(PhantomData<(E, I)>, #[cfg(debug_assertions)] Location<'static>);

impl<I: ?Sized, E> Copy for Todo<I, E> {}
impl<I: ?Sized, E> Clone for Todo<I, E> {
    fn clone(&self) -> Self {
        *self
    }
}

#[track_caller]
pub const fn todo<I: Input + ?Sized, E: Error<I>>() -> Todo<I, E> {
    Todo(PhantomData, #[cfg(debug_assertions)] *Location::caller())
}

impl<'a, I, E, S> Parser<'a, I, E, S> for Todo<I, E>
where
    I: Input + ?Sized,
    E: Error<I>,
    S: 'a,
{
    type Output = ();

    fn go<M: Mode>(&self, _inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        todo!("Attempted to use an unimplemented parser")
    }

    #[cfg(debug_assertions)]
    fn details(&self) -> (&str, Location) { ("todo", self.1) }

    #[cfg(debug_assertions)]
    fn fp(&self) -> Range<Option<usize>> { None..None }

    go_extra!();
}

#[derive(Copy, Clone)]
pub struct Choice<T, O> {
    parsers: T,
    phantom: PhantomData<O>,
    #[cfg(debug_assertions)] location: Location<'static>,
}

#[track_caller]
pub const fn choice<T, O>(parsers: T) -> Choice<T, O> {
    Choice {
        parsers,
        phantom: PhantomData,
        #[cfg(debug_assertions)] location: *Location::caller(),
    }
}

macro_rules! impl_choice_for_tuple {
    () => {};
    ($head:ident $($X:ident)*) => {
        impl_choice_for_tuple!($($X)*);
        impl_choice_for_tuple!(~ $head $($X)*);
    };
    (~ $($X:ident)*) => {
        #[allow(unused_variables, non_snake_case)]
        impl<'a, I, E, S, $($X),*, O> Parser<'a, I, E, S> for Choice<($($X,)*), O>
        where
            I: Input + ?Sized,
            E: Error<I>,
            S: 'a,
            $($X: Parser<'a, I, E, S, Output = O>),*
        {
            type Output = O;

            fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
                let before = inp.save();

                let Choice { parsers: ($($X,)*), .. } = self;

                let mut err: Option<Located<E>> = None;
                $(
                    match $X.go::<M>(inp) {
                        Ok(out) => return Ok(out),
                        Err(e) => {
                            // TODO: prioritise errors
                            err = Some(match err {
                                Some(err) => err.prioritize(e, |a, b| a.merge(b)),
                                None => e,
                            });
                            inp.rewind(before);
                        },
                    };
                )*

                Err(err.unwrap_or_else(|| Located::at(inp.last_pos(), E::expected_found(None, None, inp.span_since(before)))))
            }

            #[cfg(debug_assertions)]
            fn details(&self) -> (&str, Location) { ("choice", self.location) }

            #[cfg(debug_assertions)]
            fn fp(&self) -> Range<Option<usize>> {
                todo!();
                None..None
            }

            go_extra!();
        }
    };
}

impl_choice_for_tuple!(A_ B_ C_ D_ E_ F_ G_ H_ I_ J_ K_ L_ M_ N_ O_ P_ Q_ S_ T_ U_ V_ W_ X_ Y_ Z_);
